import torch
import shutil
from os import path
from model import Brain, DQN
from engine import MortalEngine

from libriichi.stat import Stat
from libriichi.arena import OneVsThree
from config import config

class TestPlayer:
    def __init__(self,name = "test"):
        baseline_cfg = config['baseline'][name]
        device = torch.device(baseline_cfg['device'])

        state = torch.load(baseline_cfg['state_file'], map_location=torch.device('cpu'))
        cfg = state['config']
        version = cfg['control'].get('version', 1)
        conv_channels = cfg['resnet']['conv_channels']
        num_blocks = cfg['resnet']['num_blocks']
        stable_mortal = Brain(version=version, conv_channels=conv_channels, num_blocks=num_blocks).eval()
        stable_dqn = DQN(version=version).eval()
        stable_mortal.load_state_dict(state['mortal'])
        stable_dqn.load_state_dict(state['current_dqn'])
        self.baseline_engine = MortalEngine(
            stable_mortal,
            stable_dqn,
            is_oracle = False,
            version = version,
            device = device,
            enable_amp = True,
            name = 'baseline',
        )
        self.chal_version = config['control']['version']
        self.log_dir = path.abspath(config['test_play']['log_dir'])

    def test_play(self, seed_count,repeat, mortal, dqn, device):
        torch.backends.cudnn.benchmark = False
        engine_chal = MortalEngine(
            mortal,
            dqn,
            is_oracle = False,
            version = self.chal_version,
            device = device,
            enable_amp = True,
            name = 'mortal',
        )

        if path.isdir(self.log_dir):
            shutil.rmtree(self.log_dir)

        env = OneVsThree(
            disable_progress_bar = False,
            log_dir = self.log_dir,
        )
        for i in range(repeat):
            env.py_vs_py(
                challenger = engine_chal,
                champion = self.baseline_engine,
                seed_start = (10000 + i * seed_count, 2000),
                seed_count = seed_count,
            )

        stat = Stat.from_dir(self.log_dir, 'mortal')
        torch.backends.cudnn.benchmark = config['control']['enable_cudnn_benchmark']
        return stat
