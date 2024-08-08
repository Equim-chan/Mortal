import math
from torch.optim.lr_scheduler import LambdaLR

class LinearWarmUpCosineAnnealingLR(LambdaLR):
    def __init__(self, optimizer, *, peak, final, warm_up_steps, max_steps, init=1e-8, offset=0, epoch_size=0, **kwargs):
        assert peak >= final >= init >= 0
        assert max_steps >= warm_up_steps
        self.init = init
        self.peak = peak
        self.final = final
        self.warm_up_steps = warm_up_steps
        self.max_steps = max_steps
        self.offset = offset
        self.epoch_size = epoch_size
        kwargs['optimizer'] = optimizer
        kwargs['lr_lambda'] = self._step_inner
        super().__init__(**kwargs)

    def _step_inner(self, steps):
        steps += self.offset
        if self.epoch_size > 0:
            steps %= self.epoch_size
        if self.warm_up_steps > 0 and steps < self.warm_up_steps:
            return self.init + (self.peak - self.init) / self.warm_up_steps * steps
        if steps < self.max_steps:
            cos_steps = steps - self.warm_up_steps
            cos_max_steps = self.max_steps - self.warm_up_steps
            return self.final + 0.5 * (self.peak - self.final) * (1 + math.cos(cos_steps / cos_max_steps * math.pi))
        return self.final
