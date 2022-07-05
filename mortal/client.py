import prelude

import logging
import itertools
import socket
import torch
import numpy as np
import time
from os import path
from model import Brain, DQN
from player import TrainPlayer
from common import send_msg, recv_msg
from config import config

def main():
    remote = (config['online']['remote']['host'], config['online']['remote']['port'])
    device = config['control']['device']
    oracle = Brain(True, **config['resnet']).to(device).eval()
    mortal = Brain(False, **config['resnet']).to(device).eval()
    dqn = DQN().to(device)
    train_player = TrainPlayer()

    for _ in itertools.count():
        while True:
            with socket.socket() as conn:
                conn.connect(remote)
                send_msg(conn, {'type': 'get_param'})
                rsp = recv_msg(conn, map_location=device)
                if rsp['status'] == 'ok':
                    break
                time.sleep(3)
        oracle.load_state_dict(rsp['oracle'])
        mortal.load_state_dict(rsp['mortal'])
        dqn.load_state_dict(rsp['dqn'])
        logging.info('param has been updated')

        rankings, file_list = train_player.train_play(oracle, mortal, dqn, device)
        avg_rank = (rankings * np.arange(1, 5)).sum() / rankings.sum()
        avg_pt = (rankings * np.array([90, 45, 0, -135])).sum() / rankings.sum()
        logging.info(f'trainee rankings: {rankings} ({avg_rank}, {avg_pt}pt)')

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
        torch.cuda.empty_cache()

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
