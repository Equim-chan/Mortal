import torch
import numpy as np
import os
import shutil
import secrets
from os import path
from model import Brain, DQN
from engine import MortalEngine
from libriichi.stat import Stat
from libriichi.arena import OneVsThree
from config import config

class TestPlayer:
    def __init__(self):
        device = torch.device(config['baseline']['device'])

        stable_mortal = Brain(False, **config['resnet']).eval()
        stable_dqn = DQN().eval()
        state = torch.load(config['baseline']['state_file'], map_location=torch.device('cpu'))
        stable_mortal.load_state_dict(state['mortal'])
        stable_dqn.load_state_dict(state['current_dqn'])
        self.baseline_engine = MortalEngine(
            stable_mortal,
            stable_dqn,
            is_oracle = False,
            device = device,
            enable_amp = True,
            name = 'baseline',
        )
        self.log_dir = path.abspath(config['test_play']['log_dir'])

    def test_play(self, seed_count, mortal, dqn, device):
        torch.backends.cudnn.benchmark = False
        engine_chal = MortalEngine(
            mortal,
            dqn,
            is_oracle = False,
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
        env.py_vs_py(
            challenger = engine_chal,
            champion = self.baseline_engine,
            seed_start = (10000, 2000),
            seed_count = seed_count,
        )

        stat = Stat.from_dir(self.log_dir, 'mortal')
        torch.backends.cudnn.benchmark = config['control']['enable_cudnn_benchmark']
        return stat

class TrainPlayer:
    def __init__(self):
        device = torch.device(config['baseline']['device'])

        stable_mortal = Brain(False, **config['resnet']).eval()
        stable_dqn = DQN().eval()
        state = torch.load(config['baseline']['state_file'], map_location=torch.device('cpu'))
        stable_mortal.load_state_dict(state['mortal'])
        stable_dqn.load_state_dict(state['current_dqn'])
        self.baseline_engine = MortalEngine(
            stable_mortal,
            stable_dqn,
            is_oracle = False,
            device = device,
            enable_amp = True,
            name = 'baseline',
        )

        profile = os.environ.get('TRAIN_PLAY_PROFILE', 'default')
        cfg = config['train_play'][profile]
        self.log_dir = path.abspath(cfg['log_dir'])
        self.train_key = secrets.randbits(64)
        self.train_seed = 10000

        self.seed_count = cfg['games'] // 4
        self.boltzmann_epsilon = cfg['boltzmann_epsilon']
        self.boltzmann_temp = cfg['boltzmann_temp']

        self.repeats = cfg['repeats']
        self.repeat_counter = 0

    def train_play(self, oracle, mortal, dqn, device):
        torch.backends.cudnn.benchmark = False

        engine_chal = MortalEngine(
            mortal,
            dqn,
            is_oracle = False,
            stochastic_latent = False,
            boltzmann_epsilon = self.boltzmann_epsilon,
            boltzmann_temp = self.boltzmann_temp,
            device = device,
            enable_amp = True,
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

        torch.backends.cudnn.benchmark = config['control']['enable_cudnn_benchmark']
        return rankings, file_list
