use super::{Agent, BatchifiedAgent, InvisibleState};
use crate::arena::GameResult;
use crate::chi_type::ChiType;
use crate::mjai::{Event, EventExt, Metadata};
use crate::state::PlayerState;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::prelude::*;
use std::io::{BufReader, Lines};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, ensure, Context, Result};
use serde_json as json;

pub struct AkochanAgent {
    player_id: u8,
    child: Child,
    stdin: ChildStdin,
    stdout_lines: Lines<BufReader<ChildStdout>>,

    event_idx: usize,
    naki_tx: Option<Event>,
}

impl AkochanAgent {
    pub fn new(player_id: u8) -> Result<Self> {
        ensure!(matches!(player_id, 0..=3));

        let akochan_dir = env::var_os("AKOCHAN_DIR").unwrap_or_else(|| OsString::from("akochan"));
        let akochan_exe = [&akochan_dir, OsStr::new("system.exe")]
            .iter()
            .collect::<PathBuf>();
        let akochan_tactics =
            env::var_os("AKOCHAN_TACTICS").unwrap_or_else(|| OsString::from("tactics.json"));

        let mut child = Command::new(akochan_exe)
            .arg("pipe")
            .arg(akochan_tactics)
            .arg(&player_id.to_string())
            .current_dir(akochan_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn akochan")?;

        let stdin = child
            .stdin
            .take()
            .context("failed to get stdin of akochan")?;
        let stdout = child
            .stdout
            .take()
            .context("failed to get stdout of akochan")?;
        let stdout_lines = BufReader::new(stdout).lines();

        Ok(Self {
            player_id,
            child,
            stdin,
            stdout_lines,

            event_idx: 0,
            naki_tx: None,
        })
    }

    pub fn new_batched(player_ids: &[u8]) -> Result<BatchifiedAgent<Self>> {
        BatchifiedAgent::new(Self::new, player_ids)
    }

    fn react_inner(&mut self, events: &[EventExt]) -> Result<EventExt> {
        // handle two-phase actions like Chi, Pon and Riichi
        if let Some(dahai) = self.naki_tx.take() {
            let last = events.last().context("events is empty")?;
            match last.event {
                Event::Chi { actor, .. }
                | Event::Pon { actor, .. }
                | Event::Daiminkan { actor, .. }
                | Event::Reach { actor, .. }
                    if actor == self.player_id =>
                {
                    return Ok(EventExt::no_meta(dahai));
                }
                _ => (),
            };
        }

        let start = Instant::now();
        for i in self.event_idx..events.len() {
            let ev = &events[i];

            let mut v = json::to_value(ev)?;
            if i < events.len() - 1 {
                let obj = v.as_object_mut().context("not an object")?;
                obj.insert("can_act".to_owned(), json::Value::Bool(false));
            }

            writeln!(self.stdin, "{}", json::to_string(&v)?)?;
            self.stdin.flush()?;
        }
        self.event_idx = events.len();

        let line = self
            .stdout_lines
            .next()
            .context("failed to read from akochan: unexpected EOF")?
            .context("failed to read from akochan")?;
        let actions: Vec<Event> =
            json::from_str(&line).context("failed to parse JSON output of akochan")?;
        let mut actions_iter = actions.into_iter();

        let ev = actions_iter.next().context("output is empty")?;
        if let Some(naki_tx) = actions_iter.next() {
            self.naki_tx = Some(naki_tx);
        }

        let eval_time_ns = Instant::now()
            .checked_duration_since(start)
            .unwrap_or(Duration::ZERO)
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX);
        Ok(EventExt {
            event: ev,
            meta: Some(Metadata {
                eval_time_ns: Some(eval_time_ns),
                ..Default::default()
            }),
        })
    }
}

impl Drop for AkochanAgent {
    fn drop(&mut self) {
        if let Err(err) = self.child.kill() {
            log::error!("failed to kill akochan: {err}");
        }
        if let Err(err) = self.child.wait() {
            log::error!("failed to wait akochan: {err}");
        }
    }
}

impl Agent for AkochanAgent {
    fn name(&self) -> String {
        "akochan".to_owned()
    }

    fn react(
        &mut self,
        events: &[EventExt],
        state: &PlayerState,
        _: Option<InvisibleState>,
    ) -> Result<EventExt> {
        let cans = state.last_cans();
        let ev = self.react_inner(events)?;

        match ev.event {
            Event::Dahai { pai, .. } => {
                ensure!(cans.can_discard);
                ensure!(state.discard_candidates()[pai.deaka().as_usize()]);
            }
            Event::Chi { pai, consumed, .. } => match ChiType::new(&consumed, pai) {
                ChiType::Low => ensure!(cans.can_chi_low),
                ChiType::Mid => ensure!(cans.can_chi_mid),
                ChiType::High => ensure!(cans.can_chi_high),
            },
            Event::Pon { .. } => ensure!(cans.can_pon),
            Event::Daiminkan { .. } => ensure!(cans.can_daiminkan),
            Event::Kakan { .. } => ensure!(cans.can_kakan),
            Event::Ankan { .. } => ensure!(cans.can_ankan),
            Event::Reach { .. } => ensure!(cans.can_riichi),
            Event::Hora {
                ref actor,
                ref target,
                ..
            } => {
                if actor == target {
                    ensure!(cans.can_tsumo_agari);
                } else {
                    ensure!(cans.can_ron_agari);
                }
            }
            Event::Ryukyoku { .. } => ensure!(cans.can_ryukyoku),
            Event::None => {
                ensure!(cans.can_chi() || cans.can_pon || cans.can_daiminkan || cans.can_ron_agari);
            }
            _ => bail!("unexpected response: {:?}", ev.event),
        }

        Ok(ev)
    }

    fn start_game(&mut self) -> Result<()> {
        let start_game = json::json!({
            "type": "start_game",
            "kyoku_first": 0,
            "aka_flag": true,
        });
        writeln!(self.stdin, "{}", json::to_string(&start_game)?)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn end_kyoku(&mut self) -> Result<()> {
        writeln!(self.stdin, "{}", json::to_string(&Event::EndKyoku)?)?;
        self.stdin.flush()?;
        self.event_idx = 0;
        self.naki_tx = None;
        Ok(())
    }

    fn end_game(&mut self, _: &GameResult) -> Result<()> {
        writeln!(self.stdin, "{}", json::to_string(&Event::EndGame)?)?;
        self.stdin.flush()?;
        Ok(())
    }
}
