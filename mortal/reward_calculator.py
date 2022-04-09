import torch
import numpy as np

class RewardCalculator:
    def __init__(self, grp=None, pts=None, uniform_init=False, use_score=False):
        self.device = torch.device('cpu')
        self.use_score = use_score
        if not use_score:
            self.grp = grp.to(self.device).eval()
            self.uniform_init = uniform_init

            self.pts = pts or [3, 1, -1, -3]
            self.pts = torch.tensor(pts, dtype=torch.float64, device=self.device)

    def calc_delta_grp(self, player_id, grp_feature, rank_by_player, final_scores):
        final_ranking = torch.zeros((1, 4), device=self.device)
        final_ranking[0, rank_by_player[player_id]] = 1.

        seq = list(map(
            lambda idx: torch.as_tensor(grp_feature[:idx+1], device=self.device),
            range(len(grp_feature)),
        ))

        with torch.no_grad():
            logits = self.grp(seq)
        matrix = self.grp.calc_matrix(logits)

        rank_prob = torch.cat((matrix[:, player_id], final_ranking))
        if self.uniform_init:
            rank_prob[0, :] = 1 / 4

        exp_pts = rank_prob @ self.pts
        reward = exp_pts[1:] - exp_pts[:-1]
        return reward.cpu().numpy()

    def calc_delta_points(self, player_id, grp_feature, rank_by_player, final_scores):
        seq = np.concatenate((grp_feature[:, 3 + player_id] * 1e5, final_scores[player_id]))
        delta_points = seq[1:] - seq[:-1]
        return delta_points

    def __call__(self, *args, **kwargs):
        if self.use_score:
            return self.calc_delta_points(*args, **kwargs)
        else:
            return self.calc_delta_grp(*args, **kwargs)
