def gen():
    import prelude

    import logging
    import torch
    import numpy as np
    from datetime import datetime
    from os import path
    from glob import glob
    from torch import optim
    from torch.cuda import amp
    from torch.nn import functional as F
    from torch.distributions import Normal, kl_divergence
    from torch.utils.data import DataLoader
    from torch.utils.tensorboard import SummaryWriter
    from tqdm.auto import tqdm
    from common import submit_param, parameter_count, drain
    from player import TestPlayer
    from dataloader import FileDatasetsIter, worker_init_fn
    from model import Brain, DQN
    from config import config

    device = torch.device(config["control"]["device"])
    version = config['control']['version']

    torch.backends.cudnn.benchmark = config["control"]["enable_cudnn_benchmark"]
    enable_amp = config["control"]["enable_amp"]

    mortal = Brain(version=version, **config['resnet']).to(device)
    current_dqn = DQN(version=version).to(device)

    logging.info(f"mortal params: {parameter_count(mortal):,}")
    logging.info(f"dqn params: {parameter_count(current_dqn):,}")

    mortal.freeze_bn(config["freeze_bn"]["mortal"])

    optimizer = optim.Adam(
        [
            {"params": mortal.parameters()},
            {"params": current_dqn.parameters()},
        ]
    )
    scaler = amp.GradScaler(enabled=enable_amp)

    steps = 0
    state_file = config["control"]["state_file"]

    optimizer.param_groups[0]['lr'] = config['optim']['mortal_lr']
    optimizer.param_groups[1]['lr'] = config['optim']['dqn_lr']
    optimizer.zero_grad(set_to_none=True)
    state = {
        "mortal": mortal.state_dict(),
        "current_dqn": current_dqn.state_dict(),
        "optimizer": optimizer.state_dict(),
        "scaler": scaler.state_dict(),
        "steps": steps,
        "timestamp": datetime.now().timestamp(),
        "config": config,
    }
    torch.save(state, state_file)


def main():
    gen()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
