import prelude

import os
import sys
import torch
from model import Brain, DQN
from engine import MortalEngine
from common import filtered_stripped_lines
from libriichi import Bot
from config import config

usage = '''Usage: python mortal.py <ID>

ARGS:
    <ID>    The player ID, an integer within [0, 3].'''

def main():
    try:
        player_id = int(sys.argv[-1])
        assert player_id in range(4)
    except:
        print(usage, file=sys.stderr)
        sys.exit(1)
    review_mode = os.environ.get('MORTAL_REVIEW_MODE', '0') == '1'

    device = torch.device('cpu')
    mortal = Brain(False, **config['resnet']).eval()
    dqn = DQN().eval()
    state = torch.load(config['control']['state_file'], map_location=torch.device('cpu'))
    mortal.load_state_dict(state['mortal'])
    dqn.load_state_dict(state['current_dqn'])

    engine = MortalEngine(
        mortal,
        dqn,
        is_oracle = False,
        device = device,
        enable_amp = False,
        enable_rule_based_agari_guard = True,
        name = 'mortal',
    )
    bot = Bot(engine, player_id)

    for line in filtered_stripped_lines(sys.stdin):
        if review_mode:
            print(bot.review(line), flush=True)
        elif reaction := bot.react(line):
            print(reaction, flush=True)

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
