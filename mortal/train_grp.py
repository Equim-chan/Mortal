import prelude

import random
import torch
import logging
from os import path
from glob import glob
from datetime import datetime
from torch import optim
from torch.nn import functional as F
from torch.nn.utils.rnn import pack_padded_sequence, pad_sequence
from torch.utils.data import DataLoader, IterableDataset
from torch.utils.tensorboard import SummaryWriter
from model import GRP
from libriichi.dataset import Grp
from common import tqdm
from config import config

class GrpFileDatasetsIter(IterableDataset):
    def __init__(
        self,
        file_list,
        file_batch_size = 50,
        cycle = False
    ):
        super().__init__()
        self.file_list = file_list
        self.file_batch_size = file_batch_size
        self.cycle = cycle
        self.buffer = []
        self.iterator = None

    def build_iter(self):
        while True:
            random.shuffle(self.file_list)
            for start_idx in range(0, len(self.file_list), self.file_batch_size):
                self.populate_buffer(start_idx)
                buffer_size = len(self.buffer)
                for i in random.sample(range(buffer_size), buffer_size):
                    yield self.buffer[i]
                self.buffer.clear()
            if not self.cycle:
                break

    def populate_buffer(self, start_idx):
        file_list = self.file_list[start_idx:start_idx + self.file_batch_size]
        data = Grp.load_gz_log_files(file_list)

        for game in data:
            feature = game.take_feature()
            rank_by_player = game.take_rank_by_player()

            for i in range(feature.shape[0]):
                inputs_seq = torch.as_tensor(feature[:i + 1], dtype=torch.float64)
                self.buffer.append((
                    inputs_seq,
                    rank_by_player,
                ))

    def __iter__(self):
        if self.iterator is None:
            self.iterator = self.build_iter()
        return self.iterator

def collate(batch):
    inputs = []
    lengths = []
    rank_by_players = []
    for inputs_seq, rank_by_player in batch:
        inputs.append(inputs_seq)
        lengths.append(len(inputs_seq))
        rank_by_players.append(rank_by_player)

    lengths = torch.tensor(lengths)
    rank_by_players = torch.tensor(rank_by_players, dtype=torch.int64, pin_memory=True)

    padded = pad_sequence(inputs, batch_first=True)
    packed_inputs = pack_padded_sequence(padded, lengths, batch_first=True, enforce_sorted=False)
    packed_inputs.pin_memory()

    return packed_inputs, rank_by_players

def train():
    cfg = config['grp']
    batch_size = cfg['control']['batch_size']
    save_every = cfg['control']['save_every']
    val_steps = cfg['control']['val_steps']

    device = torch.device(cfg['control']['device'])
    torch.backends.cudnn.benchmark = cfg['control']['enable_cudnn_benchmark']
    if device.type == 'cuda':
        logging.info(f'device: {device} ({torch.cuda.get_device_name(device)})')
    else:
        logging.info(f'device: {device}')

    grp = GRP(**cfg['network']).to(device)
    optimizer = optim.AdamW(grp.parameters())

    state_file = cfg['state_file']
    if path.exists(state_file):
        state = torch.load(state_file, weights_only=True, map_location=device)
        timestamp = datetime.fromtimestamp(state['timestamp']).strftime('%Y-%m-%d %H:%M:%S')
        logging.info(f'loaded: {timestamp}')
        grp.load_state_dict(state['model'])
        optimizer.load_state_dict(state['optimizer'])
        steps = state['steps']
    else:
        steps = 0

    lr = cfg['optim']['lr']
    optimizer.param_groups[0]['lr'] = lr

    file_index = cfg['dataset']['file_index']
    train_globs = cfg['dataset']['train_globs']
    val_globs = cfg['dataset']['val_globs']
    if path.exists(file_index):
        index = torch.load(file_index, weights_only=True)
        train_file_list = index['train_file_list']
        val_file_list = index['val_file_list']
    else:
        logging.info('building file index...')
        train_file_list = []
        val_file_list = []
        for pat in train_globs:
            train_file_list.extend(glob(pat, recursive=True))
        for pat in val_globs:
            val_file_list.extend(glob(pat, recursive=True))
        train_file_list.sort(reverse=True)
        val_file_list.sort(reverse=True)
        torch.save({'train_file_list': train_file_list, 'val_file_list': val_file_list}, file_index)
    writer = SummaryWriter(cfg['control']['tensorboard_dir'])

    train_file_data = GrpFileDatasetsIter(
        file_list = train_file_list,
        file_batch_size = cfg['dataset']['file_batch_size'],
        cycle = True,
    )
    train_data_loader = iter(DataLoader(
        dataset = train_file_data,
        batch_size = batch_size,
        drop_last = True,
        num_workers = 1,
        collate_fn = collate,
    ))

    val_file_data = GrpFileDatasetsIter(
        file_list = val_file_list,
        file_batch_size = cfg['dataset']['file_batch_size'],
        cycle = True,
    )
    val_data_loader = iter(DataLoader(
        dataset = val_file_data,
        batch_size = batch_size,
        drop_last = True,
        num_workers = 1,
        collate_fn = collate,
    ))

    stats = {
        'train_loss': 0,
        'train_acc': 0,
        'val_loss': 0,
        'val_acc': 0,
    }
    logging.info(f'train file list size: {len(train_file_list):,}')
    logging.info(f'val file list size: {len(val_file_list):,}')

    approx_percent = steps * batch_size / (len(train_file_list) * 10) * 100
    logging.info(f'total steps: {steps:,} est. {approx_percent:6.3f}%')

    pb = tqdm(total=save_every, desc='TRAIN')
    for inputs, rank_by_players in train_data_loader:
        inputs = inputs.to(dtype=torch.float64, device=device)
        rank_by_players = rank_by_players.to(dtype=torch.int64, device=device)

        logits = grp.forward_packed(inputs)
        labels = grp.get_label(rank_by_players)
        loss = F.cross_entropy(logits, labels)

        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        optimizer.step()

        with torch.inference_mode():
            stats['train_loss'] += loss
            stats['train_acc'] += (logits.argmax(-1) == labels).to(torch.float64).mean()

        steps += 1
        pb.update(1)

        if steps % save_every == 0:
            pb.close()

            with torch.inference_mode():
                grp.eval()
                pb = tqdm(total=val_steps, desc='VAL')
                for idx, (inputs, rank_by_players) in enumerate(val_data_loader):
                    if idx == val_steps:
                        break
                    inputs = inputs.to(dtype=torch.float64, device=device)
                    rank_by_players = rank_by_players.to(dtype=torch.int64, device=device)

                    logits = grp.forward_packed(inputs)
                    labels = grp.get_label(rank_by_players)
                    loss = F.cross_entropy(logits, labels)

                    stats['val_loss'] += loss
                    stats['val_acc'] += (logits.argmax(-1) == labels).to(torch.float64).mean()
                    pb.update(1)
                pb.close()
                grp.train()

            writer.add_scalars('loss', {
                'train': stats['train_loss'] / save_every,
                'val': stats['val_loss'] / val_steps,
            }, steps)
            writer.add_scalars('acc', {
                'train': stats['train_acc'] / save_every,
                'val': stats['val_acc'] / val_steps,
            }, steps)
            writer.add_scalar('lr', lr, steps)
            writer.flush()

            for k in stats:
                stats[k] = 0
            approx_percent = steps * batch_size / (len(train_file_list) * 10) * 100
            logging.info(f'total steps: {steps:,} est. {approx_percent:6.3f}%')

            state = {
                'model': grp.state_dict(),
                'optimizer': optimizer.state_dict(),
                'steps': steps,
                'timestamp': datetime.now().timestamp(),
            }
            torch.save(state, state_file)
            pb = tqdm(total=save_every, desc='TRAIN')
    pb.close()

if __name__ == '__main__':
    try:
        train()
    except KeyboardInterrupt:
        pass
