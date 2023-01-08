import torch
import socket
import struct
import time
from typing import *
from io import BytesIO
from functools import partial
from tqdm.auto import tqdm as orig_tqdm
from config import config

tqdm = partial(orig_tqdm, unit='batch', dynamic_ncols=True, ascii=True)

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

def filtered_trimmed_lines(lines):
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

def submit_param(oracle, mortal, dqn, is_idle=False):
    remote = (config['online']['remote']['host'], config['online']['remote']['port'])
    with socket.socket() as conn:
        conn.connect(remote)
        send_msg(conn, {
            'type': 'submit_param',
            'oracle': None if oracle is None else oracle.state_dict(),
            'mortal': mortal.state_dict(),
            'dqn': dqn.state_dict(),
            'is_idle': is_idle,
        })

def send_msg(conn: socket.socket, msg, packed=False):
    if packed:
        tx = msg
    else:
        buf = BytesIO()
        torch.save(msg, buf)
        tx = buf.getbuffer()
    conn.sendall(struct.pack('<Q', len(tx)))
    conn.sendall(tx)

def recv_msg(conn: socket.socket, map_location=torch.device('cpu')):
    rx = recv_binary(conn, 8)
    (size,) = struct.unpack('<Q', rx)
    rx = recv_binary(conn, size)
    return torch.load(BytesIO(rx), map_location=map_location)

def recv_binary(conn: socket.socket, size):
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
