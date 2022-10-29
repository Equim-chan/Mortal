import torch


@torch.jit.script
def apply_masks(actions, masks, fill: float = -1e9):
    fill = torch.tensor(fill, dtype=actions.dtype, device=actions.device)
    return torch.where(masks, actions, fill)

@torch.jit.script
def normal_kl_div(mu_p, logsig_p, mu_q, logsig_q):
    # KL(N(\mu_p, \sigma_p^2) \| N(\mu_q, \sigma_q^2)) = \log \frac{\sigma_q}{\sigma_p} + \frac{\sigma_p^2 + (\mu_p - \mu_q)^2}{2 \sigma_q^2} - \frac{1}{2}
    return logsig_q - logsig_p + \
        ((2 * logsig_p).exp() + (mu_p - mu_q) ** 2) / \
        (2 * (2 * logsig_q).exp()) - \
        0.5

def hard_update(src, dst):
    dst.load_state_dict(src.state_dict())

def parameter_count(module):
    return sum(p.numel() for p in module.parameters() if p.requires_grad)

def filtered_stripped_lines(lines):
    return filter(lambda l: l, map(lambda l: l.strip(), lines))

def iter_grads(parameters, take=False):
    for p in parameters:
        if p.grad is not None:
            if take:
                # Set to zero instead of None to preserve the layout and make it
                # easier to assign back later
                yield p.grad.clone()
                p.grad.zero_()
            else:
                yield p.grad