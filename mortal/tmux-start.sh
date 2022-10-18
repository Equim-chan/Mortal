#!/usr/bin/bash

session="mortal-train"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'background'
panle=0
tmux send-keys -t $session:$window.$panle "tensorboard --logdir /home/huguang/mortal-data/tensorboard/ --bind_all --load_fast false" C-m
tmux split-window -f -v -t $session:$window.$panle
panle=1
tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
tmux send-keys -t $session:$window.$panle "python3 ./server.py" C-m
tmux split-window -f -v -t $session:$window.$panle
panle=2
tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
tmux send-keys -t $session:$window.$panle "python3 ./rotate.py" C-m
tmux select-layout tiled

window=$(($window + 1))
tmux new-window -t $session:$window -n 'train'
panle=0
for i in {1..2}
do
tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
tmux send-keys -t $session:$window.$panle "python3 ./client.py" C-m
sleep 1
tmux split-window -f -v -t $session:$window.$panle
panle=$(($panle + 1))
done
tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
tmux send-keys -t $session:$window.$panle "python3 ./train_no_oracle.py" C-m
tmux select-layout tiled

tmux attach -t $session
