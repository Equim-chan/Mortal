import torch
from torch import nn
from torch.nn import functional as F
from torch.nn.utils.rnn import pack_padded_sequence, pad_sequence
from typing import *
from itertools import permutations
from libriichi.consts import OBS_SHAPE, ORACLE_OBS_SHAPE, ACTION_SPACE, GRP_SIZE
from common import apply_masks

class ChannelAttention(nn.Module):
    def __init__(self, channels, ratio=16):
        super().__init__()
        self.avg = nn.AdaptiveAvgPool1d(1)
        self.max = nn.AdaptiveMaxPool1d(1)
        self.shared_mlp = nn.Sequential(
            nn.Linear(channels, channels // ratio),
            nn.ReLU(inplace=True),
            nn.Linear(channels // ratio, channels),
        )

    def forward(self, x):
        avg_out = self.avg(x).squeeze(-1)
        max_out = self.max(x).squeeze(-1)
        avg_out = self.shared_mlp(avg_out)
        max_out = self.shared_mlp(max_out)
        out = torch.sigmoid(avg_out + max_out).unsqueeze(-1)
        return out

class ResBlock(nn.Module):
    def __init__(self, channels, enable_bn, bn_momentum):
        super().__init__()
        tch_bn_momentum = None
        if bn_momentum is not None:
            tch_bn_momentum = 1 - bn_momentum

        self.res_unit = nn.Sequential(
            nn.Conv1d(channels, channels, kernel_size=3, padding=1, bias=not enable_bn),
            nn.BatchNorm1d(channels, momentum=tch_bn_momentum) if enable_bn else nn.Identity(),
            nn.ReLU(inplace=True),
            nn.Conv1d(channels, channels, kernel_size=3, padding=1, bias=not enable_bn),
            nn.BatchNorm1d(channels, momentum=tch_bn_momentum) if enable_bn else nn.Identity(),
        )
        self.ca = ChannelAttention(channels)

    def forward(self, x):
        out = self.res_unit(x)
        out = self.ca(out) * out
        out += x
        out = F.relu(out, inplace=True)
        return out

class ResNet(nn.Module):
    def __init__(self, in_channels, conv_channels, num_blocks, enable_bn, bn_momentum):
        super().__init__()
        tch_bn_momentum = None
        if bn_momentum is not None:
            tch_bn_momentum = 1 - bn_momentum

        blocks = []
        for _ in range(num_blocks):
            blocks.append(ResBlock(conv_channels, enable_bn=enable_bn, bn_momentum=bn_momentum))

        self.net = nn.Sequential(
            nn.Conv1d(in_channels, conv_channels, kernel_size=3, padding=1, bias=not enable_bn),
            nn.BatchNorm1d(conv_channels, momentum=tch_bn_momentum) if enable_bn else nn.Identity(),
            nn.ReLU(inplace=True),
            *blocks,
            nn.Conv1d(conv_channels, 32, kernel_size=3, padding=1),
            nn.ReLU(inplace=True),
            nn.Flatten(),
            nn.Linear(32 * 34, 1024),
        )

    def forward(self, x):
        return self.net(x)

class Brain(nn.Module):
    def __init__(self, is_oracle, conv_channels, num_blocks, enable_bn, bn_momentum):
        super().__init__()
        self.is_oracle = is_oracle
        in_channels = OBS_SHAPE[0]
        if is_oracle:
            in_channels += ORACLE_OBS_SHAPE[0]

        if bn_momentum == 0:
            bn_momentum = None
        self.encoder = ResNet(
            in_channels,
            conv_channels = conv_channels,
            num_blocks = num_blocks,
            enable_bn = enable_bn,
            bn_momentum = bn_momentum,
        )

        self.latent_net = nn.Sequential(
            nn.Linear(1024, 512),
            nn.ReLU(inplace=True),
        )
        self.mu_head = nn.Linear(512, 512)
        self.logsig_head = nn.Linear(512, 512)

        # when True, never updates running stats, weights and bias and always use EMA or CMA
        self._freeze_bn = False

    def forward(self, obs, invisible_obs: Optional[torch.Tensor] = None):
        if self.is_oracle:
            assert invisible_obs is not None
            obs = torch.cat((obs, invisible_obs), dim=1)

        encoded = self.encoder(obs)
        latent_out = self.latent_net(encoded)
        mu = self.mu_head(latent_out)
        logsig = self.logsig_head(latent_out)
        return mu, logsig

    def train(self, mode=True):
        super().train(mode)
        if self._freeze_bn:
            for module in self.modules():
                if isinstance(module, nn.BatchNorm1d):
                    module.eval()
                    # I don't think this benefits
                    # module.requires_grad_(False)
        return self

    def reset_running_stats(self):
        for module in self.modules():
            if isinstance(module, nn.BatchNorm1d):
                module.reset_running_stats()

    def freeze_bn(self, flag):
        self._freeze_bn = flag
        return self.train(self.training)

class DQN(nn.Module):
    def __init__(self):
        super().__init__()
        self.v_head = nn.Linear(512, 1)
        self.a_head = nn.Linear(512, ACTION_SPACE)

    def forward(self, latent, mask):
        v = self.v_head(latent)
        a = self.a_head(latent)

        a_sum = apply_masks(a, mask, fill=0.).sum(-1, keepdim=True)
        mask_sum = mask.sum(-1, keepdim=True)
        a_mean = a_sum / mask_sum
        q = apply_masks(v + a - a_mean, mask)
        return q

class GRP(nn.Module):
    def __init__(self, hidden_size=64, num_layers=2):
        super().__init__()
        self.rnn = nn.GRU(input_size=GRP_SIZE, hidden_size=hidden_size, num_layers=num_layers, batch_first=True)
        self.fc = nn.Sequential(
            nn.Linear(hidden_size * num_layers, hidden_size * num_layers),
            nn.ReLU(inplace=True),
            nn.Linear(hidden_size * num_layers, 24),
        )
        for mod in self.modules():
            mod.to(torch.float64)

        # perms are the permutations of all possible rank-by-player result
        perms = torch.tensor(list(permutations(range(4))))
        perms_t = perms.transpose(0, 1)
        self.register_buffer('perms', perms)     # (24, 4)
        self.register_buffer('perms_t', perms_t) # (4, 24)

    # input: [grand_kyoku, honba, kyotaku, s[0], s[1], s[2], s[3]]
    # grand_kyoku: E1 = 0, S4 = 7, W4 = 11
    # s is 2.5 at E1
    # s[0] is score of player id 0
    def forward(self, inputs):
        lengths = torch.tensor([t.shape[0] for t in inputs], dtype=torch.int64)
        inputs = pad_sequence(inputs, batch_first=True)
        packed_inputs = pack_padded_sequence(inputs, lengths, batch_first=True, enforce_sorted=False)
        return self.forward_packed(packed_inputs)

    def forward_packed(self, packed_inputs):
        _, state = self.rnn(packed_inputs)
        state = state.transpose(0, 1).flatten(1)
        logits = self.fc(state)
        return logits

    # returns (N, player, rank_prob)
    def calc_matrix(self, logits):
        batch_size = logits.shape[0]
        probs = logits.softmax(-1)
        matrix = torch.zeros(batch_size, 4, 4, dtype=probs.dtype)
        for player in range(4):
            for rank in range(4):
                cond = self.perms_t[player] == rank
                matrix[:, player, rank] = probs[:, cond].sum(-1)
        return matrix

    def get_label(self, rank_by_player):
        batch_size = rank_by_player.shape[0]
        perms = self.perms.expand(batch_size, -1, -1).transpose(0, 1)
        mappings = (perms == rank_by_player).all(-1).nonzero()

        labels = torch.zeros(batch_size, dtype=torch.int64, device=mappings.device)
        labels[mappings[:, 1]] = mappings[:, 0]
        return labels
