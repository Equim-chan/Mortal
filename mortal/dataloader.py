import random
import torch
import numpy as np
from torch.utils.data import IterableDataset
from model import GRP
from reward_calculator import RewardCalculator
from libriichi.dataset import GameplayLoader
from config import config

class FileDatasetsIter(IterableDataset):
    def __init__(
        self,
        file_list,
        pts,
        file_batch_size = 20, # hint: around 650 instances per file
        quality_threshold = 0,
        player_name = None,
        excludes = None,
    ):
        super().__init__()
        self.file_list = file_list
        self.pts = pts
        self.file_batch_size = file_batch_size
        self.quality_threshold = int(quality_threshold)
        self.player_name = player_name
        self.excludes = excludes
        self.buffer = []
        self.iterator = None

    def build_iter(self):
        self.loader = GameplayLoader(oracle=True, player_name=self.player_name, excludes=self.excludes)

        # do not put it in __init__, it won't work on Windows
        grp = GRP(**config['grp']['network'])
        grp_state = torch.load(config['grp']['state_file'], map_location=torch.device('cpu'))
        grp.load_state_dict(grp_state['model'])
        self.reward_calc = RewardCalculator(grp, self.pts)

        random.shuffle(self.file_list)
        for start_idx in range(0, len(self.file_list), self.file_batch_size):
            self.populate_buffer(start_idx)
            buffer_size = len(self.buffer)
            for i in random.sample(range(buffer_size), buffer_size):
                yield self.buffer[i]
            self.buffer.clear()

    def populate_buffer(self, start_idx):
        file_list = self.file_list[start_idx:start_idx + self.file_batch_size]
        data = self.loader.load_gz_log_files(file_list)

        for game in data:
            quality = game.take_quality()
            if int(quality) < self.quality_threshold:
                continue

            obs = game.take_obs()
            invisible_obs = game.take_invisible_obs()
            actions = game.take_actions()
            masks = game.take_masks()
            at_kyoku = game.take_at_kyoku()
            dones = game.take_dones()
            apply_gamma = game.take_apply_gamma()
            grp = game.take_grp()
            player_id = game.take_player_id()
            game_size = len(obs)

            grp_feature = grp.take_feature()
            rank_by_player = grp.take_rank_by_player()
            final_scores = grp.take_final_scores()
            kyoku_rewards = self.reward_calc(player_id, grp_feature, rank_by_player, final_scores)

            steps_to_done = np.zeros(game_size, dtype=np.int64)
            for i in reversed(range(game_size)):
                if not dones[i]:
                    steps_to_done[i] = steps_to_done[i + 1] + int(apply_gamma[i])

            for i in range(game_size):
                entry = (
                    obs[i],
                    invisible_obs[i],
                    actions[i],
                    masks[i],
                    steps_to_done[i],
                    kyoku_rewards[at_kyoku[i]],
                )
                self.buffer.append(entry)

    def __iter__(self):
        if self.iterator is None:
            self.iterator = self.build_iter()
        return self.iterator

def worker_init_fn(*args, **kwargs):
    worker_info = torch.utils.data.get_worker_info()
    dataset = worker_info.dataset
    per_worker = int(np.ceil(len(dataset.file_list) / worker_info.num_workers))
    start = worker_info.id * per_worker
    end = start + per_worker
    dataset.file_list = dataset.file_list[start:end]
