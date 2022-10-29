
import prelude

import logging
import socket
import torch
import numpy as np
import time
import gc
from os import path,environ

from model import Brain, DQN
from player_online import TrainPlayer
from net_emit import send_msg,recv_msg

def get_config(remote):
    while True:
        with socket.socket() as conn:
            conn.connect(remote)
            send_msg(conn, {'type': 'get_test_config'})
            rsp = recv_msg(conn, map_location=torch.device('cpu'))
            if rsp['status'] == 'ok':
                return rsp['cfg']
            time.sleep(3)

def get_remote():
    ip=environ.get('MORTAL_SERVER_ADDR', '127.0.0.1')
    port=int(environ.get('MORTAL_SERVER_PORT', '5000'))
    return (ip,port)

def main():
    remote = get_remote()
    config = get_config(remote)
    logging.info('config has been loaded')
    device = torch.device(config['control']['device'])
    version = config['control']['version']
    num_blocks = config['resnet']['num_blocks']
    conv_channels = config['resnet']['conv_channels']
    oracle = None
    # oracle = Brain(version=version, is_oracle=True, num_blocks=num_blocks, conv_channels=conv_channels).to(device).eval()
    mortal = Brain(version=version, num_blocks=num_blocks, conv_channels=conv_channels).to(device).eval()
    dqn = DQN(version=version).to(device)

    while True:
        train_player = TrainPlayer(remote,version)
        while True:
            with socket.socket() as conn:
                conn.connect(remote)
                send_msg(conn, {'type': 'get_param'})
                rsp = recv_msg(conn, map_location=device)
                if rsp['status'] == 'ok':
                    break
                time.sleep(3)
        mortal.load_state_dict(rsp['mortal'])
        dqn.load_state_dict(rsp['dqn'])
        logging.info('param has been updated')
        try:
            rankings, file_list = train_player.train_play(oracle, mortal, dqn, device)
            avg_rank = (rankings * np.arange(1, 5)).sum() / rankings.sum()
            avg_pt = (rankings * np.array([90, 45, 0, -135])).sum() / rankings.sum()
            logging.info(f'trainee rankings: {rankings} ({avg_rank:.6}, {avg_pt:.6}pt)')
            logs = {}
            for filename in file_list:
                with open(filename, 'rb') as f:
                    logs[path.basename(filename)] = f.read()
            with socket.socket() as conn:
                conn.connect(remote)
                send_msg(conn, {
                    'type': 'submit_replay',
                    'logs': logs,
                })
                logging.info('logs have been submitted')
        except Exception as e:
            logging.exception('failed to game：%s',str(e))
            train_player.train_seed+=train_player.seed_count

        gc.collect()
        torch.cuda.empty_cache()
        torch.cuda.synchronize()

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
