def train():
    import prelude

    import logging
    import sys
    import os
    import gc
    import random
    import torch
    import numpy as np
    from datetime import datetime
    from os import path
    from glob import glob
    from torch import optim
    from torch.cuda.amp import GradScaler
    from torch.nn import functional as F
    from torch.nn.utils import clip_grad_norm_
    from torch.distributions import Normal
    from torch.utils.data import DataLoader
    from torch.utils.tensorboard import SummaryWriter
    from tqdm.auto import tqdm
    from common import normal_kl_div, submit_param, parameter_count, drain, filtered_trimmed_lines
    from player import TestPlayer
    from dataloader import FileDatasetsIter, worker_init_fn
    from model import Brain, DQN
    from config import config

    version = config['control']['version']

    online = config['control']['online']
    batch_size = config['control']['batch_size']
    opt_step_every = config['control']['opt_step_every']
    save_every = config['control']['save_every']
    test_every = config['control']['test_every']
    test_games = config['test_play']['games']
    log10_kld_target = np.log10(config['vlog']['kld_target'])
    free_bits_threshold = config['vlog']['free_bits_threshold']
    free_bits_weight = config['vlog']['free_bits_weight']
    min_q_weight = config['cql']['min_q_weight']
    assert save_every % opt_step_every == 0
    assert test_every % save_every == 0

    device = torch.device(config['control']['device'])
    torch.backends.cudnn.benchmark = config['control']['enable_cudnn_benchmark']
    enable_amp = config['control']['enable_amp']

    pts = config['env']['pts']
    gamma = config['env']['gamma']
    file_batch_size = config['dataset']['file_batch_size']
    num_workers = config['dataset']['num_workers']
    eps = config['optim']['eps']
    weight_decay = config['optim']['weight_decay']
    max_grad_norm = config['optim']['max_grad_norm']

    mortal = Brain(version=version, **config['resnet']).to(device)
    current_oracle = Brain(is_oracle=True, version=version, **config['resnet']).to(device)
    current_dqn = DQN(version=version).to(device)
    log_beta = torch.tensor(config['vlog']['beta_init'], dtype=torch.float32, device=device).log().requires_grad_(True)

    logging.info(f'mortal params: {parameter_count(mortal):,}')
    logging.info(f'oracle params: {parameter_count(current_oracle):,}')
    logging.info(f'dqn params: {parameter_count(current_dqn):,}')

    mortal.freeze_bn(config['freeze_bn']['mortal'])
    current_oracle.freeze_bn(config['freeze_bn']['oracle'])

    param_groups = [
        {'params': mortal.parameters()},
        {'params': current_oracle.parameters()},
        {'params': current_dqn.parameters()},
        {'params': [log_beta]},
    ]
    optimizer = optim.AdamW(param_groups, eps=eps, weight_decay=weight_decay)
    scaler = GradScaler(enabled=enable_amp)
    test_player = TestPlayer()

    steps = 0
    state_file = config['control']['state_file']
    if path.exists(state_file):
        state = torch.load(state_file, map_location=device)
        timestamp = datetime.fromtimestamp(state['timestamp']).strftime('%Y-%m-%d %H:%M:%S')
        logging.info(f'loaded: {timestamp}')
        mortal.load_state_dict(state['mortal'])
        current_oracle.load_state_dict(state['current_oracle'])
        current_dqn.load_state_dict(state['current_dqn'])
        log_beta.detach().copy_(state['log_beta'])
        optimizer.load_state_dict(state['optimizer'])
        scaler.load_state_dict(state['scaler'])
        steps = state['steps']

    optimizer.param_groups[0]['lr'] = config['optim']['mortal_lr']
    optimizer.param_groups[1]['lr'] = config['optim']['oracle_lr']
    optimizer.param_groups[2]['lr'] = config['optim']['dqn_lr']
    optimizer.param_groups[3]['lr'] = config['optim']['beta_lr']
    optimizer.zero_grad(set_to_none=True)

    if device.type == 'cuda':
        logging.info(f'device: {device} ({torch.cuda.get_device_name(device)})')
    else:
        logging.info(f'device: {device}')

    if online:
        submit_param(current_oracle, mortal, current_dqn)
        logging.info('param has been submitted')

    writer = SummaryWriter(config['control']['tensorboard_dir'])
    stats = {
        'dqn_loss': 0,
        'cql_loss': 0,
        'kld_loss': 0,
        'free_bits_loss': 0,
        'beta_loss': 0,
        'mortal_entropy': 0,
        'oracle_entropy': 0,
    }
    all_q = torch.zeros((save_every, batch_size), device=device, dtype=torch.float32)
    all_q_target = torch.zeros((save_every, batch_size), device=device, dtype=torch.float32)
    idx = 0

    def train_epoch():
        nonlocal steps
        nonlocal idx

        player_names = []
        if online:
            player_names = ['trainee']
            dirname = drain()
            file_list = list(map(lambda p: path.join(dirname, p), os.listdir(dirname)))
        else:
            for filename in config['dataset']['player_names_files']:
                with open(filename) as f:
                    player_names.extend(filtered_trimmed_lines(f))
            player_names = list(set(player_names))
            logging.info(f'loaded {len(player_names):,} players')

            file_index = config['dataset']['file_index']
            if path.exists(file_index):
                index = torch.load(file_index)
                file_list = index['file_list']
            else:
                logging.info('building file index...')
                file_list = []
                for pat in config['dataset']['globs']:
                    file_list.extend(glob(pat, recursive=True))
                file_list.sort(reverse=True)
                torch.save({'file_list': file_list}, file_index)
        logging.info(f'file list size: {len(file_list):,}')

        before_next_test_play = (test_every - steps % test_every) % test_every
        logging.info(f'total steps: {steps:,} (~{before_next_test_play:,})')

        def build_data_loader(augmented):
            random.shuffle(file_list)
            file_data = FileDatasetsIter(
                version = version,
                file_list = file_list,
                pts = pts,
                file_batch_size = file_batch_size,
                player_names = player_names,
                augmented = augmented,
            )
            return iter(DataLoader(
                dataset = file_data,
                batch_size = batch_size,
                drop_last = True,
                num_workers = num_workers,
                pin_memory = True,
                worker_init_fn = worker_init_fn,
            ))
        data_loader = build_data_loader(False)
        if enable_augment:
            augmented = build_data_loader(True)
            data_loader = chain.from_iterable(zip(data_loader, augmented))

        pb = tqdm(total=save_every, desc='TRAIN', unit='batch', dynamic_ncols=True, ascii=True)
        for obs, invisible_obs, actions, masks, steps_to_done, kyoku_rewards in data_loader:
            obs = obs.to(dtype=torch.float32, device=device)
            invisible_obs = invisible_obs.to(dtype=torch.float32, device=device)
            actions = actions.to(dtype=torch.int64, device=device)
            masks = masks.to(dtype=torch.bool, device=device)
            steps_to_done = steps_to_done.to(dtype=torch.int64, device=device)
            kyoku_rewards = kyoku_rewards.to(dtype=torch.float64, device=device)
            assert masks[range(batch_size), actions].all()

            q_target_mc = gamma ** steps_to_done * kyoku_rewards
            q_target_mc = q_target_mc.to(torch.float32)

            with torch.autocast(device.type, enabled=enable_amp):
                mu, logsig = current_oracle(obs, invisible_obs)
                dist = Normal(mu, logsig.exp() + 1e-6)
                latent = dist.rsample()

                q_out = current_dqn(latent, masks)
                q = q_out[range(batch_size), actions]
                dqn_loss = 0.5 * F.mse_loss(q, q_target_mc)

                mu_mortal, logsig_mortal = mortal(obs)
                dist_mortal = Normal(mu_mortal, logsig_mortal.exp() + 1e-6)
                kld = normal_kl_div(mu, logsig, mu_mortal, logsig_mortal)
                kld_loss = kld.sum(-1).mean()
                free_bits_loss = kld.clamp(free_bits_threshold).sum(-1).mean()
                beta_loss = log_beta * (log10_kld_target - kld_loss.detach().log10())

                cql_loss = 0
                if not online:
                    cql_loss = q_out.logsumexp(-1).mean() - q.mean()

                loss = sum((
                    dqn_loss,
                    cql_loss * min_q_weight,
                    kld_loss * log_beta.detach().exp(),
                    free_bits_loss * free_bits_weight,
                    beta_loss,
                )) / opt_step_every
            scaler.scale(loss).backward()

            with torch.no_grad():
                stats['dqn_loss'] += dqn_loss
                if not online:
                    stats['cql_loss'] += cql_loss
                stats['kld_loss'] += kld_loss
                stats['free_bits_loss'] += free_bits_loss - 512 * free_bits_threshold
                stats['beta_loss'] += beta_loss
                stats['mortal_entropy'] += dist_mortal.entropy().sum(-1).mean()
                stats['oracle_entropy'] += dist.entropy().sum(-1).mean()
                all_q[idx] = q
                all_q_target[idx] = q_target_mc

            steps += 1
            idx += 1
            if idx % opt_step_every == 0:
                if max_grad_norm > 0:
                    scaler.unscale_(optimizer)
                    for group in optimizer.param_groups:
                        clip_grad_norm_(group['params'], max_grad_norm)
                scaler.step(optimizer)
                scaler.update()
                optimizer.zero_grad(set_to_none=True)
            pb.update(1)

            if steps % save_every == 0:
                pb.close()

                # downsample to reduce tensorboard event size
                all_q_1d = all_q.cpu().numpy().flatten()[::128]
                all_q_target_1d = all_q_target.cpu().numpy().flatten()[::128]

                writer.add_scalar('loss/dqn_loss', stats['dqn_loss'] / save_every, steps)
                if not online:
                    writer.add_scalar('loss/cql_loss', stats['cql_loss'] / save_every, steps)
                writer.add_scalar('loss/kld_loss', stats['kld_loss'] / save_every, steps)
                writer.add_scalar('loss/free_bits_loss', stats['free_bits_loss'] / save_every, steps)
                writer.add_scalar('loss/beta_loss', stats['beta_loss'] / save_every, steps)
                writer.add_scalar('vlog/beta', log_beta.detach().exp(), steps)
                writer.add_scalar('vlog/mortal_entropy', stats['mortal_entropy'] / save_every, steps)
                writer.add_scalar('vlog/oracle_entropy', stats['oracle_entropy'] / save_every, steps)
                writer.add_histogram('q_predicted', all_q_1d, steps)
                writer.add_histogram('q_target', all_q_target_1d, steps)
                writer.flush()

                for k in stats:
                    stats[k] = 0
                idx = 0

                before_next_test_play = (test_every - steps % test_every) % test_every
                logging.info(f'total steps: {steps:,} (~{before_next_test_play:,})')

                state = {
                    'mortal': mortal.state_dict(),
                    'current_oracle': current_oracle.state_dict(),
                    'current_dqn': current_dqn.state_dict(),
                    'log_beta': log_beta,
                    'optimizer': optimizer.state_dict(),
                    'scaler': scaler.state_dict(),
                    'steps': steps,
                    'timestamp': datetime.now().timestamp(),
                    'config': config,
                }
                torch.save(state, state_file)

                if online:
                    submit_param(current_oracle, mortal, current_dqn)
                    logging.info('param has been submitted')

                if steps % test_every == 0:
                    stat = test_player.test_play(test_games // 4, mortal, current_dqn, device)
                    mortal.train()
                    current_dqn.train()

                    avg_pt = stat.avg_pt([90, 45, 0, -135])
                    logging.info(f'avg rank: {stat.avg_rank:.6}')
                    logging.info(f'avg pt: {avg_pt:.6}')
                    writer.add_scalar('test_play/avg_ranking', stat.avg_rank, steps)
                    writer.add_scalar('test_play/avg_pt', avg_pt, steps)
                    writer.add_scalars('test_play/ranking', {
                        '1st': stat.rank_1_rate,
                        '2nd': stat.rank_2_rate,
                        '3rd': stat.rank_3_rate,
                        '4th': stat.rank_4_rate,
                    }, steps)
                    writer.add_scalars('test_play/behavior', {
                        'agari': stat.agari_rate,
                        'houjuu': stat.houjuu_rate,
                        'fuuro': stat.fuuro_rate,
                        'riichi': stat.riichi_rate,
                    }, steps)
                    writer.add_scalars('test_play/agari_point', {
                        'overall': stat.avg_point_per_agari,
                        'riichi': stat.avg_point_per_riichi_agari,
                        'fuuro': stat.avg_point_per_fuuro_agari,
                        'dama': stat.avg_point_per_dama_agari,
                    }, steps)
                    writer.add_scalar('test_play/houjuu_point', stat.avg_point_per_houjuu, steps)
                    writer.add_scalar('test_play/point_per_round', stat.avg_point_per_round, steps)
                    writer.add_scalars('test_play/key_step', {
                        'agari_jun': stat.avg_agari_jun,
                        'houjuu_jun': stat.avg_houjuu_jun,
                        'riichi_jun': stat.avg_riichi_jun,
                    }, steps)
                    writer.add_scalars('test_play/riichi', {
                        'agari_after_riichi': stat.agari_rate_after_riichi,
                        'houjuu_after_riichi': stat.houjuu_rate_after_riichi,
                        'chasing_riichi': stat.chasing_riichi_rate,
                        'riichi_chased': stat.riichi_chased_rate,
                    }, steps)
                    writer.add_scalar('test_play/riichi_point', stat.avg_riichi_point, steps)
                    writer.add_scalars('test_play/fuuro', {
                        'agari_after_fuuro': stat.agari_rate_after_fuuro,
                        'houjuu_after_fuuro': stat.houjuu_rate_after_fuuro,
                    }, steps)
                    writer.add_scalar('test_play/fuuro_num', stat.avg_fuuro_num, steps)
                    writer.add_scalar('test_play/fuuro_point', stat.avg_fuuro_point, steps)
                    writer.flush()
                    if online:
                        # BUG: This is an bug with unkown reason. When training
                        # in online mode, the process will get stuck here. This
                        # is the reason why `main` spawns a sub process to train
                        # in online mode instead of going for training directly.
                        sys.exit(0)

                pb = tqdm(total=save_every, desc='TRAIN', unit='batch', dynamic_ncols=True, ascii=True)
        pb.close()

        if online:
            submit_param(current_oracle, mortal, current_dqn)
            logging.info('param has been submitted')

    while True:
        train_epoch()
        gc.collect()
        torch.cuda.empty_cache()
        torch.cuda.synchronize()
        if not online:
            # only run one epoch for offline for easier control
            break

def main():
    import os
    import sys
    import time
    from subprocess import Popen
    from config import config

    # do not set this env manually
    is_sub_proc_key = 'MORTAL_IS_SUB_PROC'
    online = config['control']['online']
    if not online or os.environ.get(is_sub_proc_key, '0') == '1':
        train()
        return

    cmd = (sys.executable, __file__)
    env = {
        is_sub_proc_key: '1',
        **os.environ.copy(),
    }
    while True:
        child = Popen(
            cmd,
            stdin = sys.stdin,
            stdout = sys.stdout,
            stderr = sys.stderr,
            env = env,
        )
        if (code := child.wait()) != 0:
            sys.exit(code)
        time.sleep(3)

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
