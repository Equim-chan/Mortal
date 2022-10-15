#!/usr/bin/bash

session="mortal-train"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'background'
tmux send-keys -t $session:$window "tensorboard --logdir /home/huguang/mortal-data/tensorboard/ --bind_all --load_fast false" C-m
tmux split-window
tmux send-keys -t $session:$window "pyenv activate mortal" C-m
tmux send-keys -t $session:$window "python3 ./server.py" C-m
tmux split-window
tmux send-keys -t $session:$window "pyenv activate mortal" C-m
tmux send-keys -t $session:$window "python3 ./rotate.py" C-m
tmux select-layout tiled

window=1
tmux new-window -t $session:$window -n 'train'
tmux send-keys -t $session:$window "pyenv activate mortal" C-m
tmux send-keys -t $session:$window "TRAIN_PLAY_PROFILE=self python3 ./client.py" C-m
tmux split-window
tmux send-keys -t $session:$window "pyenv activate mortal" C-m
tmux send-keys -t $session:$window "python3 ./client.py" C-m
tmux split-window
tmux send-keys -t $session:$window "pyenv activate mortal" C-m
tmux send-keys -t $session:$window "python3 ./client.py" C-m
tmux split-window
tmux send-keys -t $session:$window "pyenv activate mortal" C-m
tmux send-keys -t $session:$window "python3 ./train_no_oracle.py" C-m
tmux select-layout tiled


tmux attach -t $session
