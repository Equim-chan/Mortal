# Strength
The Mortal snapshot used in this is `22040703`.

## Simulation environment
The simulation uses a duplicate mahjong setting as follows

| Table | East start (0) | South start (1) | West start (2) | North start (3) |
|:---:|:---:|:---:|:---:|:---:|
| 1 | Player A | Player B | Player B | Player B |
| 2 | Player B | Player A | Player B | Player B |
| 3 | Player B | Player B | Player A | Player B |
| 4 | Player B | Player B | Player B | Player A |

In this setting, every 4 games are initialized with same random seed. The emulator guarantees that with the same (seed, kyoku, honba) tuple, the yama, haipai, dora/ura indicators and rinshan tiles are deterministic and reproducible.

The emulator is implemented in [`libriichi::arena`](https://github.com/Equim-chan/Mortal/tree/main/libriichi/src/arena).

## Mortal vs akochan
Player A is akochan and Player B is Mortal.

### akochan with jun_pt = [90, 45, 0, -135]
|  | Mortal | akochan |
|---:|:---|:---|
| Games | 70296 | 23432 |
| Rounds | 750504 | 250168 |
| Rounds as dealer | 188838 | 61330 |
|  |  |  |
| 1st (rate) | 17850 (0.253926) | 5582 (0.238221) |
| 2nd (rate) | 17911 (0.254794) | 5521 (0.235618) |
| 3rd (rate) | 17538 (0.249488) | 5894 (0.251536) |
| 4th (rate) | 16997 (0.241792) | 6435 (0.274624) |
| Tobi(rate) | 4426 (0.062962) | 1916 (0.081769) |
| Avg rank | 2.479145 | 2.562564 |
| Total pt | 117900 | -117900 |
| Avg game pt | 1.677194 | -5.031581 |
| Total Δscore | 28694400 | -28694400 |
| Avg game Δscore | 408.193923 | -1224.581769 |
| Avg round Δscore | 38.233507 | -114.700521 |
|  |  |  |
| Win rate | 0.213570 | 0.195952 |
| Deal-in rate | 0.114841 | 0.131907 |
| Call rate | 0.296321 | 0.333208 |
| Riichi rate | 0.181443 | 0.215923 |
| Ryukyoku rate | 0.168335 | 0.168335 |
|  |  |  |
| Avg winning Δscore | 6483.038962 | 6747.283817 |
| Avg winning Δscore as dealer | 8439.922489 | 8812.331701 |
| Avg winning Δscore as non-dealer | 5727.687944 | 5996.378202 |
| Avg riichi winning Δscore | 8079.942256 | 8271.697489 |
| Avg open winning Δscore | 4976.946811 | 4976.754839 |
| Avg dama winning Δscore | 6515.832517 | 7745.257453 |
| Avg ryukyoku Δscore | -25.851697 | 77.555091 |
|  |  |  |
| Avg winning turn | 11.129126 | 11.182881 |
| Avg riichi winning turn | 11.126644 | 11.249845 |
| Avg open winning turn | 11.086393 | 11.117438 |
| Avg dama winning turn | 11.264727 | 11.120403 |
|  |  |  |
| Avg deal-in turn | 11.485306 | 11.617261 |
| Avg deal-in Δscore | -5332.635255 | -5334.579836 |
| Avg deal-in Δscore to dealer | -7104.268622 | -7075.531317 |
| Avg deal-in Δscore to non-dealer | -4716.800350 | -4692.321414 |
|  |  |  |
| Chasing riichi rate | 0.180357 | 0.148842 |
| Riichi chased rate | 0.175775 | 0.182350 |
| Winning rate after riichi | 0.485805 | 0.447470 |
| Deal-in rate after riichi | 0.149147 | 0.159913 |
| Avg riichi turn | 7.798390 | 7.944443 |
| Avg riichi Δscore | 3207.900187 | 2916.183794 |
|  |  |  |
| Avg number of calls | 1.449355 | 1.433600 |
| Winning rate after call | 0.317703 | 0.267125 |
| Deal-in rate after call | 0.131733 | 0.145337 |
| Avg call Δscore | 907.807905 | 562.586674 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.236388 | 0.213142 |
| Dealer wins/all wins | 0.278498 | 0.266661 |
| Deal-in to dealer/all deal-ins | 0.257945 | 0.269493 |
|  |  |  |
| Yakuman (rate) | 112 (0.000149233) | 21 (0.000083944) |
| Nagashi mangan (rate) | 20 (0.000026649) | 0 (0.000000000) |

### akochan with jun_pt = [90, 30, -30, -90]
|  | Mortal | akochan |
|---:|:---|:---|
| Games | 73164 | 24388 |
| Rounds | 779958 | 259986 |
| Rounds as dealer | 196050 | 63936 |
|  |  |  |
| 1st (rate) | 18341 (0.250683) | 6047 (0.247950) |
| 2nd (rate) | 18501 (0.252870) | 5887 (0.241389) |
| 3rd (rate) | 18918 (0.258570) | 5470 (0.224291) |
| 4th (rate) | 17404 (0.237877) | 6984 (0.286370) |
| Tobi(rate) | 4580 (0.062599) | 2058 (0.084386) |
| Avg rank | 2.483639 | 2.549082 |
| Total pt | 133695 | -133695 |
| Avg game pt | 1.827333 | -5.481999 |
| Total Δscore | 24085000 | -24085000 |
| Avg game Δscore | 329.191952 | -987.575857 |
| Avg round Δscore | 30.879868 | -92.639604 |
|  |  |  |
| Win rate | 0.213935 | 0.197126 |
| Deal-in rate | 0.114030 | 0.138531 |
| Call rate | 0.296377 | 0.331276 |
| Riichi rate | 0.180537 | 0.225578 |
| Ryukyoku rate | 0.166147 | 0.166147 |
|  |  |  |
| Avg winning Δscore | 6442.207240 | 6887.453659 |
| Avg winning Δscore as dealer | 8355.842889 | 9107.688304 |
| Avg winning Δscore as non-dealer | 5710.956124 | 6097.529166 |
| Avg riichi winning Δscore | 8015.720543 | 8426.118884 |
| Avg open winning Δscore | 4977.606459 | 5089.079611 |
| Avg dama winning Δscore | 6456.614746 | 7552.749577 |
| Avg ryukyoku Δscore | -56.008272 | 168.024817 |
|  |  |  |
| Avg winning turn | 11.082213 | 11.229112 |
| Avg riichi winning turn | 11.068438 | 11.295316 |
| Avg open winning turn | 11.056963 | 11.173257 |
| Avg dama winning turn | 11.199211 | 11.048223 |
|  |  |  |
| Avg deal-in turn | 11.451006 | 11.619114 |
| Avg deal-in Δscore | -5345.643643 | -5321.343292 |
| Avg deal-in Δscore to dealer | -7139.180226 | -6970.939034 |
| Avg deal-in Δscore to non-dealer | -4734.051536 | -4706.768293 |
|  |  |  |
| Chasing riichi rate | 0.176882 | 0.166999 |
| Riichi chased rate | 0.183203 | 0.173581 |
| Winning rate after riichi | 0.488023 | 0.441182 |
| Deal-in rate after riichi | 0.150216 | 0.159309 |
| Avg riichi turn | 7.780905 | 8.046447 |
| Avg riichi Δscore | 3181.566071 | 2947.654611 |
|  |  |  |
| Avg number of calls | 1.453448 | 1.444425 |
| Winning rate after call | 0.320407 | 0.267187 |
| Deal-in rate after call | 0.131492 | 0.153808 |
| Avg call Δscore | 914.957043 | 564.244662 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.235312 | 0.210351 |
| Dealer wins/all wins | 0.276477 | 0.262420 |
| Deal-in to dealer/all deal-ins | 0.254287 | 0.271435 |
|  |  |  |
| Yakuman (rate) | 163 (0.000208986) | 31 (0.000119237) |
| Nagashi mangan (rate) | 15 (0.000019232) | 0 (0.000000000) |


## Tenhou
Tenhou does not allow AI to play in ranked lobby without their permission, therefore I only compared how closely Mortal's decisions matched those of other verified AIs that had previously played in Tenhou tokujou. I also sampled some tokujou and houou games to check against Mortal.

[![](../assets/match-rate.png)](../assets/match-rate.png)
