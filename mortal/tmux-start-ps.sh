#!/usr/bin/bash

py="python3"

session="mortal-train"

log_dir=$($py -c "from config import config ;print(config['control']['tensorboard_dir'])")

log_base_dir=$(dirname $log_dir)

tmux new-session -d -s $session

{
    window=0
    tmux rename-window -t $session:$window 'background'
    panle=0
    tmux send-keys -t $session:$window.$panle "tensorboard --logdir $log_base_dir --bind_all --load_fast false" C-m
    tmux split-window -f -v -t $session:$window.$panle
    panle=1
    tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
    tmux send-keys -t $session:$window.$panle "$py ./server.py" C-m
    tmux split-window -f -v -t $session:$window.$panle
    panle=2
    tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
    tmux send-keys -t $session:$window.$panle "$py ./rotate.py" C-m
    tmux select-layout tiled

    sleep 5

    window=$(($window + 1))
    tmux new-window -t $session:$window -n 'train'
    panle=0
    tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
    tmux send-keys -t $session:$window.$panle "$py ./train_no_oracle.py" C-m
}&

tmux attach -t $session
