def main():
    import prelude

    import logging
    import sys
    from model import Brain, DQN, NextRankPredictor
    from config import config
    import torch
    from common import parameter_count
    from libriichi.consts import obs_shape
    from os import path
    from datetime import datetime
    from torch import optim, nn
    from lr_scheduler import LinearWarmUpCosineAnnealingLR
    from torch.cuda.amp import GradScaler


    version = config['control']['version']
    online = config['control']['online']
    device = torch.device(config['control']['device'])
    weight_decay = config['optim']['weight_decay']
    enable_amp = config['control']['enable_amp']
    eps = config['optim']['eps']
    betas = config['optim']['betas']

    mortal = Brain(version=version, **config['resnet']).to(device)
    current_dqn = DQN(version=version).to(device)
    next_rank_pred = NextRankPredictor().to(device)

    logging.info(f'version: {version}')
    logging.info(f'mortal params: {parameter_count(mortal):,}')
    logging.info(f'dqn params: {parameter_count(current_dqn):,}')
    logging.info(f'next_rank_pred params: {parameter_count(next_rank_pred):,}')
    logging.info(f'obs shape: {obs_shape(version)}')

    decay_params = []
    no_decay_params = []
    for model in (mortal, current_dqn):
        params_dict = {}
        to_decay = set()
        for mod_name, mod in model.named_modules():
            for name, param in mod.named_parameters(prefix=mod_name, recurse=False):
                params_dict[name] = param
                if isinstance(mod, (nn.Linear, nn.Conv1d)) and name.endswith('weight'):
                    to_decay.add(name)
        decay_params.extend(params_dict[name] for name in sorted(to_decay))
        no_decay_params.extend(params_dict[name] for name in sorted(params_dict.keys() - to_decay))
    param_groups = [
        {'params': decay_params, 'weight_decay': weight_decay},
        {'params': no_decay_params},
    ]
    optimizer = optim.AdamW(param_groups, lr=1, weight_decay=0, betas=betas, eps=eps)
    scheduler = LinearWarmUpCosineAnnealingLR(optimizer, **config['optim']['scheduler'])
    scaler = GradScaler(enabled=enable_amp)


    state_file = config['control']['state_file']
    best_state_file = config['control']['best_state_file']


    if path.exists(best_state_file):
        logging.info(f'best_state_file file still exists. Please properly backup and remove.')
        sys.exit(0)


    if path.exists(state_file):
        state = torch.load(state_file, map_location=device)
        timestamp = state['timestamp']
        mortal.load_state_dict(state['mortal'])
        current_dqn.load_state_dict(state['current_dqn'])
        next_rank_pred.load_state_dict(state['next_rank_pred'])
        optimizer.load_state_dict(state['optimizer'])
        scheduler.load_state_dict(state['scheduler'])
        scaler.load_state_dict(state['scaler'])
        steps = state['steps']
        best_perf = {
            'avg_rank': 4.,
            'avg_pt': -135.,
        }

        state = {
            'mortal': mortal.state_dict(),
            'current_dqn': current_dqn.state_dict(),
            'next_rank_pred': next_rank_pred.state_dict(),
            'optimizer': optimizer.state_dict(),
            'scheduler': scheduler.state_dict(),
            'scaler': scaler.state_dict(),
            'steps': steps,
            'timestamp': timestamp,
            'best_perf': best_perf,
            'config': config,
        }

        torch.save(state, state_file)
        logging.info(f'best_perf successfully resetted.')

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
