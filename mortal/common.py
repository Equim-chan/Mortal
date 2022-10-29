import socket
import time
from typing import *
from collections import OrderedDict
from io import BytesIO
from config import config
from net_emit import send_msg,recv_msg

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
            'oracle': None if oracle is None else oracle.state_dict(),
            'mortal': mortal.state_dict(),
            'dqn': dqn.state_dict(),
        })

