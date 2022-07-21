import torch
import socket
import struct
import time
from typing import *
from io import BytesIO
from config import config

@torch.jit.script
def apply_masks(actions, masks, fill: float = -1e9):
    fill = torch.tensor(fill, dtype=actions.dtype, device=actions.device)
    return torch.where(masks, actions, fill)

@torch.jit.script
def normal_kl_div(mu_p, logsig_p, mu_q, logsig_q):
    # KL(N(\mu_p, \sigma_p) \| N(\mu_q, \sigma_q)) = \log \frac{\sigma_q}{\sigma_p} + \frac{\sigma_p^2 + (\mu_p - \mu_q)^2}{2 \sigma_q^2} - \frac{1}{2}
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

def drain():
    remote = (config['online']['remote']['host'], config['online']['remote']['port'])
    while True:
        with socket.socket() as conn:
            conn.connect(remote)
            send_msg(conn, {'type': 'drain'})
            msg = recv_msg(conn)
        if msg['count'] == 0:
            time.sleep(5)
            continue
        return msg['drain_dir']

def submit_param(oracle, mortal, dqn):
    remote = (config['online']['remote']['host'], config['online']['remote']['port'])
    with socket.socket() as conn:
        conn.connect(remote)
        send_msg(conn, {
            'type': 'submit_param',
            'oracle': oracle.state_dict(),
            'mortal': mortal.state_dict(),
            'dqn': dqn.state_dict(),
        })

def send_msg(conn, msg, packed=False):
    if packed:
        tx = msg
    else:
        buf = BytesIO()
        torch.save(msg, buf)
        tx = buf.getvalue()
    conn.sendall(struct.pack('<Q', len(tx)))
    conn.sendall(tx)

def recv_msg(conn, map_location=torch.device('cpu')):
    rx = recv_binary(conn, 8)
    (size,) = struct.unpack('<Q', rx)
    rx = recv_binary(conn, size)
    return torch.load(BytesIO(rx), map_location=map_location)

def recv_binary(conn, size):
    assert size > 0
    ret = bytearray(size)
    buf = memoryview(ret)

    while len(buf) > 0:
        n = conn.recv_into(buf)
        if n == 0:
            raise UnexpectedEOF()
        buf = buf[n:]
    return bytes(ret)

class UnexpectedEOF(Exception):
    def __init__(self):
        super().__init__('unexpected EOF')
