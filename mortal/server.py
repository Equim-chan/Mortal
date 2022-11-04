import prelude

import logging
import random
import shutil
import torch
import sys
import os
from os import path
from io import BytesIO
from socketserver import ThreadingTCPServer, BaseRequestHandler
from threading import Lock
from net_emit import send_msg, recv_msg, UnexpectedEOF
from config import config

buffer_dir = path.abspath(config['online']['server']['buffer_dir'])
drain_dir = path.abspath(config['online']['server']['drain_dir'])

dir_lock = Lock()
param_lock = Lock()
buffer_size = 0
oracle_param = None
mortal_param = None
dqn_param = None
sample_reuse_rate = config['online']['server']['sample_reuse_rate']
sample_reuse_threshold = config['online']['server']['sample_reuse_threshold']
capacity = config['online']['server']['capacity']

def save_log(filename, content):
    filepath = path.join(buffer_dir, filename)
    with open(filepath, 'wb') as f:
        f.write(content)

def move_log(filename):
    src = path.join(buffer_dir, filename)
    dst = path.join(drain_dir, filename)
    shutil.move(src, dst)

def delete_drain(filename):
    filepath = path.join(drain_dir, filename)
    os.remove(filepath)

def update_param(oracle, mortal, dqn):
    global oracle_param
    global mortal_param
    global dqn_param
    with param_lock:
        oracle_param = oracle
        mortal_param = mortal
        dqn_param = dqn

def set_config(msg):
    global sample_reuse_rate
    global sample_reuse_threshold
    global capacity
    with dir_lock:
        sample_reuse_rate = msg['sample_reuse_rate']
        sample_reuse_threshold = msg['sample_reuse_threshold']
        capacity = msg['capacity']

class Handler(BaseRequestHandler):
    def handle(self):
        global buffer_size
        msg = self.recv_msg()

        match msg['type']:
            case 'get_param':
                self.get_param()

            case 'get_test_param':
                self.get_test_param(msg['name'] if msg['name'] is not None else 'default')

            case 'get_test_config':
                self.get_test_config()

            case 'submit_replay':
                with dir_lock:
                    for filename, content in msg['logs'].items():
                        save_log(filename, content)
                    buffer_size += len(msg['logs'])
                    logging.info(f'total buffer size: {buffer_size}')

            case 'submit_param':
                update_param(msg['oracle'], msg['mortal'], msg['dqn'])

            case 'drain':
                with dir_lock:
                    buffer_list = os.listdir(buffer_dir)
                    count = len(buffer_list)
                    if count > 0:
                        drain_list = os.listdir(drain_dir)
                        to_delete_count = int(max(
                            len(drain_list) * (1 - sample_reuse_rate),
                            # x/(k+x) = t, x = tk/(1-t)
                            len(drain_list) - (count * sample_reuse_threshold) / (1 - sample_reuse_threshold),
                        ))
                        logging.info(f'previously drained files to delete: {to_delete_count}')
                        to_delete = random.sample(drain_list, to_delete_count)
                        for filename in to_delete:
                            delete_drain(filename)
                        for filename in buffer_list:
                            move_log(filename)

                        drain_size = len(drain_list) - to_delete_count + count
                        buffer_size = 0
                        logging.info(f'new drain files size: {drain_size}')
                        logging.info(f'total buffer size: {buffer_size}')
                self.send_msg({
                    'count': count,
                    'drain_dir': drain_dir,
                })

            case 'set_config':
                set_config(msg)
                with dir_lock:
                    logging.info(f'sample_reuse_rate = {sample_reuse_rate}')
                    logging.info(f'sample_reuse_threshold = {sample_reuse_threshold}')
                    logging.info(f'capacity = {capacity}')

    def get_param(self):
        with dir_lock:
            overflow = buffer_size >= capacity
            with param_lock:
                has_param = mortal_param is not None and dqn_param is not None
        if not has_param or overflow:
            self.send_msg({'status': 'empty param or log overflow'})
            return

        with param_lock:
            res = {
                'status': 'ok',
                'mortal': mortal_param,
                'dqn': dqn_param,
            }
            buf = BytesIO()
            packed = torch.save(res, buf)
        self.send_msg(buf.getvalue(), packed=True)

    def get_test_param(self,name):
        cfg = config['train_play'][name]
        while True:
            try:
                state = torch.load(cfg['state_file'], map_location=torch.device('cpu'))
                break
            except RuntimeError as e:
                logging.exception('failed to load state file：%s',str(e))
                pass
        res = {
            'status': 'ok',
            'mortal': state['mortal'],
            'dqn': state['current_dqn'],
            'model_cfg': state['config'],
            'cfg': cfg,
        }
        buf = BytesIO()
        torch.save(res, buf)
        self.send_msg(buf.getvalue(), packed=True)

    def get_test_config(self):
        cfg = config
        res = {
            'status': 'ok',
            'cfg': cfg,
        }
        buf = BytesIO()
        torch.save(res, buf)
        self.send_msg(buf.getvalue(), packed=True)

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
    bind_addr = (config['online']['remote']['host'], config['online']['remote']['port'])
    if path.isdir(buffer_dir):
        shutil.rmtree(buffer_dir)
    if path.isdir(drain_dir):
        shutil.rmtree(drain_dir)
    os.makedirs(buffer_dir)
    os.makedirs(drain_dir)

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
