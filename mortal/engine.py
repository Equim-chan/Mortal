import torch
import numpy as np
from torch.distributions import Normal, Categorical
from common import apply_masks

class MortalEngine:
    def __init__(
        self,
        brain,
        dqn,
        is_oracle,
        device = None,
        stochastic_latent = False,
        enable_amp = False,
        enable_quick_eval = True,
        enable_rule_based_agari_guard = False,
        name = 'NoName',
        boltzmann_epsilon = 0,
        boltzmann_temp = 1,
    ):
        self.device = device or torch.device('cpu')
        assert isinstance(self.device, torch.device)
        self.brain = brain.to(self.device).eval()
        self.dqn = dqn.to(self.device).eval()
        self.is_oracle = is_oracle
        self.stochastic_latent = stochastic_latent

        self.enable_amp = enable_amp
        self.enable_quick_eval = enable_quick_eval
        self.enable_rule_based_agari_guard = enable_rule_based_agari_guard
        self.name = name

        self.boltzmann_epsilon = boltzmann_epsilon
        self.boltzmann_temp = boltzmann_temp

    def react_batch(self, obs, masks, invisible_obs):
        with (
            torch.autocast(self.device.type, enabled=self.enable_amp),
            torch.no_grad(),
        ):
            return self._react_batch(obs, masks, invisible_obs)

    def _react_batch(self, obs, masks, invisible_obs):
        obs = torch.as_tensor(np.stack(obs, axis=0), device=self.device)
        masks = torch.as_tensor(np.stack(masks, axis=0), device=self.device)
        if self.is_oracle:
            invisible_obs = torch.as_tensor(np.stack(invisible_obs, axis=0), device=self.device)
        else:
            invisible_obs = None
        batch_size = obs.shape[0]

        mu, logsig = self.brain(obs, invisible_obs)
        if self.stochastic_latent:
            latent = Normal(mu, logsig.exp()).sample()
        else:
            latent = mu
        q_out = self.dqn(latent, masks)

        if self.boltzmann_epsilon > 0:
            is_greedy = torch.rand(batch_size, device=self.device) >= self.boltzmann_epsilon
            logits = apply_masks(q_out / self.boltzmann_temp, masks, fill=-1e9)
            actions = torch.where(
                is_greedy,
                q_out.argmax(-1),
                Categorical(logits=logits).sample(),
            )
        else:
            is_greedy = torch.ones(batch_size, dtype=torch.bool, device=self.device)
            actions = q_out.argmax(-1)

        return actions.tolist(), q_out.tolist(), masks.tolist(), is_greedy.tolist()
