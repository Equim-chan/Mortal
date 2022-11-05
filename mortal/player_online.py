
import logging
import time
import torch
import numpy as np
import os
import shutil
import socket
import secrets
from os import path
from model import Brain, DQN
from engine import MortalEngine
from net_emit import send_msg,recv_msg

from libriichi.arena import OneVsThree

class TrainPlayer:
    def __init__(self,server,version):
        profile = os.environ.get('TRAIN_PLAY_PROFILE', 'default')
        remote = server
        while True:
            with socket.socket() as conn:
                conn.connect(remote)
                send_msg(conn, {'type': 'get_test_param','name': profile})
                rsp = recv_msg(conn, map_location=torch.device('cpu'))
                logging.info('test param has been updated')
                if rsp['status'] == 'ok':
                    break
                time.sleep(3)
        cfg = rsp['cfg']
        device = torch.device(cfg['device'])

        model_cfg = rsp['model_cfg']
        version = model_cfg['control'].get('version', 1)
        conv_channels = model_cfg['resnet']['conv_channels']
        num_blocks = model_cfg['resnet']['num_blocks']
        stable_mortal = Brain(version=version, conv_channels=conv_channels, num_blocks=num_blocks).eval()
        stable_dqn = DQN(version=version).eval()
        stable_mortal.load_state_dict(rsp['mortal'])
        stable_dqn.load_state_dict(rsp['dqn'])
        self.baseline_engine = MortalEngine(
            stable_mortal,
            stable_dqn,
            is_oracle = False,
            version = version,
            device = device,
            enable_amp = True,
            name = 'baseline',
        )

        self.chal_version = version
        self.log_dir =path.join(path.abspath("/tmp/mortal"), str(os.getpid()))
        self.train_key = secrets.randbits(64)
        self.train_seed = 10000

        self.seed_count = cfg['games'] // 4
        self.boltzmann_epsilon = cfg['boltzmann_epsilon']
        self.boltzmann_temp = cfg['boltzmann_temp']
        self.stochastic_latent = cfg['stochastic_latent']

        self.repeats = cfg['repeats']
        self.repeat_counter = 0

    def train_play(self, oracle, mortal, dqn, device):
        torch.backends.cudnn.benchmark = False
        engine_chal = MortalEngine(
            mortal,
            dqn,
            is_oracle = False,
            version = self.chal_version,
            stochastic_latent = self.stochastic_latent,
            boltzmann_epsilon = self.boltzmann_epsilon,
            boltzmann_temp = self.boltzmann_temp,
            device = device,
            enable_amp = False,
            name = 'trainee',
        )

        if path.isdir(self.log_dir):
            shutil.rmtree(self.log_dir)

        env = OneVsThree(
            disable_progress_bar = False,
            log_dir = self.log_dir,
        )
        rankings = env.py_vs_py(
            challenger = engine_chal,
            champion = self.baseline_engine,
            seed_start = (self.train_seed, self.train_key),
            seed_count = self.seed_count,
        )
        self.repeat_counter += 1
        if self.repeat_counter == self.repeats:
            self.train_seed += self.seed_count
            self.repeat_counter = 0

        rankings = np.array(rankings)
        file_list = list(map(lambda p: path.join(self.log_dir, p), os.listdir(self.log_dir)))
        return rankings, file_list