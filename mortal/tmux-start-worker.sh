#!/usr/bin/bash

py="python3"

session="mortal-train"

server_ip="0.0.0.0"

tmux new-session -d -s $session

{
    window=0
    tmux new-window -t $session:$window -n 'train'
    panle=0
    count=6
    i=1
    while ((i <= $count))
    do
        tmux send-keys -t $session:$window.$panle "pyenv activate mortal" C-m
        tmux send-keys -t $session:$window.$panle " MORTAL_SERVER_ADDR=$server_ip MORTAL_SERVER_PORT=5000 TRAIN_PLAY_PROFILE=default-${i} $py ./client.py" C-m
        tmux select-layout even-vertical
        if [ $i -ne $count ]
        then
            tmux split-window -f -v -t $session:$window.$panle
            panle=$(($panle + 1))
        fi
        i=$(($i + 1))
    done
}&

tmux attach -t $session
