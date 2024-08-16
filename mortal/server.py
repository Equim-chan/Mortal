import prelude

import logging
import shutil
import torch
import sys
import os
from os import path
from io import BytesIO
from typing import *
from collections import OrderedDict
from dataclasses import dataclass
from socketserver import ThreadingTCPServer, BaseRequestHandler
from threading import Lock
from common import send_msg, recv_msg, UnexpectedEOF
from config import config

@dataclass
class State:
    buffer_dir: str
    drain_dir: str
    capacity: int
    force_sequential: bool
    dir_lock: Lock
    param_lock: Lock
    # fields below are protected by dir_lock
    buffer_size: int
    submission_id: int
    # fields below are protected by param_lock
    mortal_param: Optional[OrderedDict]
    dqn_param: Optional[OrderedDict]
    param_version: int
    idle_param_version: int
S = None

class Handler(BaseRequestHandler):
    def handle(self):
        msg = self.recv_msg()
        match msg['type']:
            # called by workers
            case 'get_param':
                self.handle_get_param(msg)
            case 'submit_replay':
                self.handle_submit_replay(msg)
            # called by trainer
            case 'submit_param':
                self.handle_submit_param(msg)
            case 'drain':
                self.handle_drain()

    def handle_get_param(self, msg):
        with S.dir_lock:
            overflow = S.buffer_size >= S.capacity
            with S.param_lock:
                has_param = S.mortal_param is not None and S.dqn_param is not None
        if overflow:
            self.send_msg({'status': 'samples overflow'})
            return
        if not has_param:
            self.send_msg({'status': 'empty param'})
            return

        client_param_version = msg['param_version']
        buf = BytesIO()
        with S.param_lock:
            if S.force_sequential and S.idle_param_version <= client_param_version:
                res = {'status': 'trainer is busy'}
            else:
                res = {
                    'status': 'ok',
                    'mortal': S.mortal_param,
                    'dqn': S.dqn_param,
                    'param_version': S.param_version,
                }
            torch.save(res, buf)
        self.send_msg(buf.getbuffer(), packed=True)

    def handle_submit_replay(self, msg):
        with S.dir_lock:
            for filename, content in msg['logs'].items():
                filepath = path.join(S.buffer_dir, f'{S.submission_id}_{filename}')
                with open(filepath, 'wb') as f:
                    f.write(content)
            S.buffer_size += len(msg['logs'])
            S.submission_id += 1
            logging.info(f'total buffer size: {S.buffer_size}')

    def handle_submit_param(self, msg):
        with S.param_lock:
            S.mortal_param = msg['mortal']
            S.dqn_param = msg['dqn']
            S.param_version += 1
            if msg['is_idle']:
                S.idle_param_version = S.param_version

    def handle_drain(self):
        drained_size = 0
        with S.dir_lock:
            buffer_list = os.listdir(S.buffer_dir)
            raw_count = len(buffer_list)
            assert raw_count == S.buffer_size
            if (not S.force_sequential or raw_count >= S.capacity) and raw_count > 0:
                old_drain_list = os.listdir(S.drain_dir)
                for filename in old_drain_list:
                    filepath = path.join(S.drain_dir, filename)
                    os.remove(filepath)
                for filename in buffer_list:
                    src = path.join(S.buffer_dir, filename)
                    dst = path.join(S.drain_dir, filename)
                    shutil.move(src, dst)
                drained_size = raw_count
                S.buffer_size = 0
                logging.info(f'files transferred to trainer: {drained_size}')
                logging.info(f'total buffer size: {S.buffer_size}')
        self.send_msg({
            'count': drained_size,
            'drain_dir': S.drain_dir,
        })

    def send_msg(self, msg, packed=False):
        return send_msg(self.request, msg, packed)

    def recv_msg(self):
        return recv_msg(self.request)

class Server(ThreadingTCPServer):
    def handle_error(self, request, client_address):
        typ, _, _ = sys.exc_info()
        if typ is BrokenPipeError or typ is UnexpectedEOF:
            return
        return super().handle_error(request, client_address)

def main():
    global S
    cfg = config['online']['server']
    S = State(
        buffer_dir = path.abspath(cfg['buffer_dir']),
        drain_dir = path.abspath(cfg['drain_dir']),
        capacity = cfg['capacity'],
        force_sequential = cfg['force_sequential'],
        dir_lock = Lock(),
        param_lock = Lock(),
        buffer_size = 0,
        submission_id = 0,
        mortal_param = None,
        dqn_param = None,
        param_version = 0,
        idle_param_version = 0,
    )

    bind_addr = (config['online']['remote']['host'], config['online']['remote']['port'])
    if path.isdir(S.buffer_dir):
        shutil.rmtree(S.buffer_dir)
    if path.isdir(S.drain_dir):
        shutil.rmtree(S.drain_dir)
    os.makedirs(S.buffer_dir)
    os.makedirs(S.drain_dir)

    with Server(bind_addr, Handler, bind_and_activate=False) as server:
        server.allow_reuse_address = True
        server.daemon_threads = True
        server.server_bind()
        server.server_activate()
        host, port = bind_addr
        logging.info(f'listening on {host}:{port}')
        server.serve_forever()

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
