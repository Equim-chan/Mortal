# Strength
## Simulation environment
The simulation uses a 1v3 duplicate mahjong setting as follows

| Table | East start | South start | West start | North start |
|:---:|:---:|:---:|:---:|:---:|
| 1 | Challenger | Champion | Champion | Champion |
| 2 | Champion | Challenger | Champion | Champion |
| 3 | Champion | Champion | Challenger | Champion |
| 4 | Champion | Champion | Champion | Challenger |

In this setting, every 4 games are initialized with same random seed. The emulator guarantees that with the same (seed, kyoku, honba) tuple, the yama, haipai, dora/ura indicators and rinshan tiles are deterministic and reproducible.

The emulator is implemented in [`libriichi::arena`](https://github.com/Equim-chan/Mortal/tree/main/libriichi/src/arena).

## Mortal vs akochan (jun_pt = [90, 45, 0, -135])
Challenger is akochan and Champion is Mortal.

### mortal1-b40c192-t22040618
<details>

|  | akochan (x1) | mortal1-b40c192-t22040618 (x3) |
|---:|:---|:---|
| Games | 23432 | 70296 |
| Rounds | 250168 | 750504 |
| Rounds as dealer | 61330 | 188838 |
|  |  |  |
| 1st (rate) | 5582 (0.238221) | 17850 (0.253926) |
| 2nd (rate) | 5521 (0.235618) | 17911 (0.254794) |
| 3rd (rate) | 5894 (0.251536) | 17538 (0.249488) |
| 4th (rate) | 6435 (0.274624) | 16997 (0.241792) |
| Tobi(rate) | 1916 (0.081769) | 4426 (0.062962) |
| Avg rank | 2.562564 | 2.479145 |
| Total pt | -117900 | 117900 |
| Avg game pt | -5.031581 | 1.677194 |
| Total Δscore | -28694400 | 28694400 |
| Avg game Δscore | -1224.581769 | 408.193923 |
| Avg round Δscore | -114.700521 | 38.233507 |
|  |  |  |
| Win rate | 0.195952 | 0.213570 |
| Deal-in rate | 0.131907 | 0.114841 |
| Call rate | 0.333208 | 0.296321 |
| Riichi rate | 0.215923 | 0.181443 |
| Ryukyoku rate | 0.168335 | 0.168335 |
|  |  |  |
| Avg winning Δscore | 6747.283817 | 6483.038962 |
| Avg winning Δscore as dealer | 8812.331701 | 8439.922489 |
| Avg winning Δscore as non-dealer | 5996.378202 | 5727.687944 |
| Avg riichi winning Δscore | 8271.697489 | 8079.942256 |
| Avg open winning Δscore | 4976.754839 | 4976.946811 |
| Avg dama winning Δscore | 7745.257453 | 6515.832517 |
| Avg ryukyoku Δscore | 77.555091 | -25.851697 |
|  |  |  |
| Avg winning turn | 11.182881 | 11.129126 |
| Avg riichi winning turn | 11.249845 | 11.126644 |
| Avg open winning turn | 11.117438 | 11.086393 |
| Avg dama winning turn | 11.120403 | 11.264727 |
|  |  |  |
| Avg deal-in turn | 11.617261 | 11.485306 |
| Avg deal-in Δscore | -5334.579836 | -5332.635255 |
| Avg deal-in Δscore to dealer | -7075.531317 | -7104.268622 |
| Avg deal-in Δscore to non-dealer | -4692.321414 | -4716.800350 |
|  |  |  |
| Chasing riichi rate | 0.148842 | 0.180357 |
| Riichi chased rate | 0.182350 | 0.175775 |
| Winning rate after riichi | 0.447470 | 0.485805 |
| Deal-in rate after riichi | 0.159913 | 0.149147 |
| Avg riichi turn | 7.944443 | 7.798390 |
| Avg riichi Δscore | 2916.183794 | 3207.900187 |
|  |  |  |
| Avg number of calls | 1.433600 | 1.449355 |
| Winning rate after call | 0.267125 | 0.317703 |
| Deal-in rate after call | 0.145337 | 0.131733 |
| Avg call Δscore | 562.586674 | 907.807905 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.213142 | 0.236388 |
| Dealer wins/all wins | 0.266661 | 0.278498 |
| Deal-in to dealer/all deal-ins | 0.269493 | 0.257945 |
|  |  |  |
| Yakuman (rate) | 21 (0.000083944) | 112 (0.000149233) |
| Nagashi mangan (rate) | 0 (0.000000000) | 20 (0.000026649) |

</details>

### mortal3-b24c512-t22122709
<details>

|  | akochan (x1) | mortal3-b24c512-t22122709 (x3) |
|---:|:---|:---|
| Games | 13152 | 39456 |
| Rounds | 140132 | 420396 |
| Rounds as dealer | 34602 | 105530 |
|  |  |  |
| 1st (rate) | 3121 (0.237302) | 10031 (0.254233) |
| 2nd (rate) | 3116 (0.236922) | 10036 (0.254359) |
| 3rd (rate) | 3243 (0.246578) | 9909 (0.251141) |
| 4th (rate) | 3672 (0.279197) | 9480 (0.240268) |
| Tobi(rate) | 1028 (0.078163) | 2439 (0.061816) |
| Avg rank | 2.567670 | 2.477443 |
| Total pt | -74610 | 74610 |
| Avg game pt | -5.672901 | 1.890967 |
| Total Δscore | -14601500 | 14601500 |
| Avg game Δscore | -1110.211375 | 370.070458 |
| Avg round Δscore | -104.198185 | 34.732728 |
|  |  |  |
| Win rate | 0.195352 | 0.212157 |
| Deal-in rate | 0.130420 | 0.112613 |
| Call rate | 0.330075 | 0.293685 |
| Riichi rate | 0.215304 | 0.189110 |
| Ryukyoku rate | 0.172801 | 0.172801 |
|  |  |  |
| Avg winning Δscore | 6807.616438 | 6510.059424 |
| Avg winning Δscore as dealer | 9027.834773 | 8508.379567 |
| Avg winning Δscore as non-dealer | 5983.888416 | 5760.331190 |
| Avg riichi winning Δscore | 8404.128236 | 8146.596125 |
| Avg open winning Δscore | 4985.047857 | 4842.311007 |
| Avg dama winning Δscore | 7606.341789 | 6749.915065 |
| Avg ryukyoku Δscore | 90.192030 | -30.064010 |
|  |  |  |
| Avg winning turn | 11.143854 | 11.182128 |
| Avg riichi winning turn | 11.242041 | 11.182500 |
| Avg open winning turn | 11.055497 | 11.142524 |
| Avg dama winning turn | 10.995995 | 11.320519 |
|  |  |  |
| Avg deal-in turn | 11.714325 | 11.457247 |
| Avg deal-in Δscore | -5299.611512 | -5284.607748 |
| Avg deal-in Δscore to dealer | -6995.398902 | -7088.703069 |
| Avg deal-in Δscore to non-dealer | -4706.123504 | -4674.846678 |
|  |  |  |
| Chasing riichi rate | 0.157370 | 0.180702 |
| Riichi chased rate | 0.183255 | 0.179620 |
| Winning rate after riichi | 0.445593 | 0.485013 |
| Deal-in rate after riichi | 0.159557 | 0.149470 |
| Avg riichi turn | 7.954758 | 7.815939 |
| Avg riichi Δscore | 2979.530012 | 3245.600684 |
|  |  |  |
| Avg number of calls | 1.441908 | 1.435309 |
| Winning rate after call | 0.268798 | 0.319494 |
| Deal-in rate after call | 0.145285 | 0.124854 |
| Avg call Δscore | 592.184460 | 914.366131 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.214092 | 0.230579 |
| Dealer wins/all wins | 0.270612 | 0.272822 |
| Deal-in to dealer/all deal-ins | 0.259247 | 0.252609 |
|  |  |  |
| Yakuman (rate) | 16 (0.000114178) | 49 (0.000116557) |
| Nagashi mangan (rate) | 0 (0.000000000) | 46 (0.000109421) |

</details>

## Mortal vs akochan (jun_pt = [90, 30, -30, -90])
### mortal1-b40c192-t22040618
<details>

|  | akochan (x1) | mortal1-b40c192-t22040618 (x3) |
|---:|:---|:---|
| Games | 24388 | 73164 |
| Rounds | 259986 | 779958 |
| Rounds as dealer | 63936 | 196050 |
|  |  |  |
| 1st (rate) | 6047 (0.247950) | 18341 (0.250683) |
| 2nd (rate) | 5887 (0.241389) | 18501 (0.252870) |
| 3rd (rate) | 5470 (0.224291) | 18918 (0.258570) |
| 4th (rate) | 6984 (0.286370) | 17404 (0.237877) |
| Tobi(rate) | 2058 (0.084386) | 4580 (0.062599) |
| Avg rank | 2.549082 | 2.483639 |
| Total pt | -133695 | 133695 |
| Avg game pt | -5.481999 | 1.827333 |
| Total Δscore | -24085000 | 24085000 |
| Avg game Δscore | -987.575857 | 329.191952 |
| Avg round Δscore | -92.639604 | 30.879868 |
|  |  |  |
| Win rate | 0.197126 | 0.213935 |
| Deal-in rate | 0.138531 | 0.114030 |
| Call rate | 0.331276 | 0.296377 |
| Riichi rate | 0.225578 | 0.180537 |
| Ryukyoku rate | 0.166147 | 0.166147 |
|  |  |  |
| Avg winning Δscore | 6887.453659 | 6442.207240 |
| Avg winning Δscore as dealer | 9107.688304 | 8355.842889 |
| Avg winning Δscore as non-dealer | 6097.529166 | 5710.956124 |
| Avg riichi winning Δscore | 8426.118884 | 8015.720543 |
| Avg open winning Δscore | 5089.079611 | 4977.606459 |
| Avg dama winning Δscore | 7552.749577 | 6456.614746 |
| Avg ryukyoku Δscore | 168.024817 | -56.008272 |
|  |  |  |
| Avg winning turn | 11.229112 | 11.082213 |
| Avg riichi winning turn | 11.295316 | 11.068438 |
| Avg open winning turn | 11.173257 | 11.056963 |
| Avg dama winning turn | 11.048223 | 11.199211 |
|  |  |  |
| Avg deal-in turn | 11.619114 | 11.451006 |
| Avg deal-in Δscore | -5321.343292 | -5345.643643 |
| Avg deal-in Δscore to dealer | -6970.939034 | -7139.180226 |
| Avg deal-in Δscore to non-dealer | -4706.768293 | -4734.051536 |
|  |  |  |
| Chasing riichi rate | 0.166999 | 0.176882 |
| Riichi chased rate | 0.173581 | 0.183203 |
| Winning rate after riichi | 0.441182 | 0.488023 |
| Deal-in rate after riichi | 0.159309 | 0.150216 |
| Avg riichi turn | 8.046447 | 7.780905 |
| Avg riichi Δscore | 2947.654611 | 3181.566071 |
|  |  |  |
| Avg number of calls | 1.444425 | 1.453448 |
| Winning rate after call | 0.267187 | 0.320407 |
| Deal-in rate after call | 0.153808 | 0.131492 |
| Avg call Δscore | 564.244662 | 914.957043 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.210351 | 0.235312 |
| Dealer wins/all wins | 0.262420 | 0.276477 |
| Deal-in to dealer/all deal-ins | 0.271435 | 0.254287 |
|  |  |  |
| Yakuman (rate) | 31 (0.000119237) | 163 (0.000208986) |
| Nagashi mangan (rate) | 0 (0.000000000) | 15 (0.000019232) |

</details>

## Mortal vs Mortal
### mortal2-b75c256-t22092920 and mortal1-b40c192-t22040618
<details>

|  | mortal2-b75c256-t22092920 (x1) | mortal1-b40c192-t22040618 (x3) |
|---:|:---|:---|
| Games | 426000 | 1278000 |
| Rounds | 4568485 | 13705455 |
| Rounds as dealer | 1130506 | 3437979 |
|  |  |  |
| 1st (rate) | 107160 (0.251549) | 318840 (0.249484) |
| 2nd (rate) | 106427 (0.249829) | 319573 (0.250057) |
| 3rd (rate) | 105945 (0.248697) | 320055 (0.250434) |
| 4th (rate) | 106468 (0.249925) | 319532 (0.250025) |
| Tobi(rate) | 27006 (0.063394) | 87330 (0.068333) |
| Avg rank | 2.496998 | 2.501001 |
| Total pt | 60435 | -60435 |
| Avg game pt | 0.141866 | -0.047289 |
| Total Δscore | 18652900 | -18652900 |
| Avg game Δscore | 43.786150 | -14.595383 |
| Avg round Δscore | 4.082951 | -1.360984 |
|  |  |  |
| Win rate | 0.201100 | 0.210842 |
| Deal-in rate | 0.113477 | 0.119711 |
| Call rate | 0.288314 | 0.295943 |
| Riichi rate | 0.173797 | 0.182667 |
| Ryukyoku rate | 0.170837 | 0.170837 |
|  |  |  |
| Avg winning Δscore | 6624.815505 | 6436.649081 |
| Avg winning Δscore as dealer | 8550.270545 | 8377.695608 |
| Avg winning Δscore as non-dealer | 5882.672301 | 5698.101070 |
| Avg riichi winning Δscore | 8235.661556 | 8026.424142 |
| Avg open winning Δscore | 5149.989008 | 4928.198868 |
| Avg dama winning Δscore | 6460.737173 | 6446.738607 |
| Avg ryukyoku Δscore | -67.458884 | 22.486295 |
|  |  |  |
| Avg winning turn | 11.073204 | 11.143723 |
| Avg riichi winning turn | 11.087846 | 11.151681 |
| Avg open winning turn | 11.010515 | 11.089835 |
| Avg dama winning turn | 11.225206 | 11.282352 |
|  |  |  |
| Avg deal-in turn | 11.502015 | 11.482273 |
| Avg deal-in Δscore | -5235.412861 | -5323.966492 |
| Avg deal-in Δscore to dealer | -6947.785537 | -7033.802381 |
| Avg deal-in Δscore to non-dealer | -4646.055777 | -4717.602231 |
|  |  |  |
| Chasing riichi rate | 0.163332 | 0.164827 |
| Riichi chased rate | 0.174174 | 0.168552 |
| Winning rate after riichi | 0.483505 | 0.479043 |
| Deal-in rate after riichi | 0.147938 | 0.149382 |
| Avg riichi turn | 7.751164 | 7.814533 |
| Avg riichi Δscore | 3278.816709 | 3131.209817 |
|  |  |  |
| Avg number of calls | 1.468083 | 1.444199 |
| Winning rate after call | 0.307361 | 0.312322 |
| Deal-in rate after call | 0.135190 | 0.136126 |
| Avg call Δscore | 920.822027 | 858.670427 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.226089 | 0.231663 |
| Dealer wins/all wins | 0.278207 | 0.275619 |
| Deal-in to dealer/all deal-ins | 0.256050 | 0.261793 |
|  |  |  |
| Yakuman (rate) | 657 (0.000143811) | 2103 (0.000153443) |
| Nagashi mangan (rate) | 92 (0.000020138) | 403 (0.000029404) |

</details>

Swapping Challenger and Champion.

<details>

|  | mortal1-b40c192-t22040618 (x1) | mortal2-b75c256-t22092920 (x3) |
|---:|:---|:---|
| Games | 404000 | 1212000 |
| Rounds | 4364061 | 13092183 |
| Rounds as dealer | 1103344 | 3260717 |
|  |  |  |
| 1st (rate) | 100909 (0.249775) | 303091 (0.250075) |
| 2nd (rate) | 101357 (0.250884) | 302643 (0.249705) |
| 3rd (rate) | 101105 (0.250260) | 302895 (0.249913) |
| 4th (rate) | 100629 (0.249082) | 303371 (0.250306) |
| Tobi(rate) | 29033 (0.071864) | 81611 (0.067336) |
| Avg rank | 2.498649 | 2.500450 |
| Total pt | 57960 | -57960 |
| Avg game pt | 0.143465 | -0.047822 |
| Total Δscore | 13969900 | -13969900 |
| Avg game Δscore | 34.578960 | -11.526320 |
| Avg round Δscore | 3.201124 | -1.067041 |
|  |  |  |
| Win rate | 0.212621 | 0.201806 |
| Deal-in rate | 0.118069 | 0.112019 |
| Call rate | 0.296154 | 0.287104 |
| Riichi rate | 0.185261 | 0.176752 |
| Ryukyoku rate | 0.186210 | 0.186210 |
|  |  |  |
| Avg winning Δscore | 6468.779812 | 6662.244567 |
| Avg winning Δscore as dealer | 8402.976332 | 8583.978730 |
| Avg winning Δscore as non-dealer | 5726.791793 | 5914.078352 |
| Avg riichi winning Δscore | 8071.102088 | 8282.872970 |
| Avg open winning Δscore | 4920.254645 | 5141.249336 |
| Avg dama winning Δscore | 6487.726056 | 6499.767462 |
| Avg ryukyoku Δscore | 69.193852 | -23.064617 |
|  |  |  |
| Avg winning turn | 11.226868 | 11.165918 |
| Avg riichi winning turn | 11.219288 | 11.171787 |
| Avg open winning turn | 11.184648 | 11.112177 |
| Avg dama winning turn | 11.369596 | 11.309909 |
|  |  |  |
| Avg deal-in turn | 11.524012 | 11.554706 |
| Avg deal-in Δscore | -5461.129451 | -5379.466500 |
| Avg deal-in Δscore to dealer | -7145.173422 | -7080.005371 |
| Avg deal-in Δscore to non-dealer | -4849.924613 | -4777.221692 |
|  |  |  |
| Chasing riichi rate | 0.162070 | 0.159654 |
| Riichi chased rate | 0.162165 | 0.166870 |
| Winning rate after riichi | 0.478268 | 0.481330 |
| Deal-in rate after riichi | 0.146050 | 0.145433 |
| Avg riichi turn | 7.864415 | 7.809567 |
| Avg riichi Δscore | 3166.434464 | 3293.847974 |
|  |  |  |
| Avg number of calls | 1.435226 | 1.459294 |
| Winning rate after call | 0.310897 | 0.304871 |
| Deal-in rate after call | 0.133552 | 0.132994 |
| Avg call Δscore | 859.858175 | 912.474713 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.233167 | 0.227058 |
| Dealer wins/all wins | 0.277256 | 0.280223 |
| Deal-in to dealer/all deal-ins | 0.266291 | 0.261529 |
|  |  |  |
| Yakuman (rate) | 675 (0.000154672) | 1843 (0.000140771) |
| Nagashi mangan (rate) | 140 (0.000032080) | 300 (0.000022914) |

</details>

### mortal2-b75c256-t22100115 and mortal1-b40c192-t22040618
<details>

|  | mortal2-b75c256-t22100115 (x1) | mortal1-b40c192-t22040618 (x3) |
|---:|:---|:---|
| Games | 88000 | 264000 |
| Rounds | 938739 | 2816217 |
| Rounds as dealer | 232809 | 705930 |
|  |  |  |
| 1st (rate) | 23142 (0.262977) | 64858 (0.245674) |
| 2nd (rate) | 21440 (0.243636) | 66560 (0.252121) |
| 3rd (rate) | 20580 (0.233864) | 67420 (0.255379) |
| 4th (rate) | 22838 (0.259523) | 65162 (0.246826) |
| Tobi(rate) | 6158 (0.069977) | 18546 (0.070250) |
| Avg rank | 2.489932 | 2.503356 |
| Total pt | -35550 | 35550 |
| Avg game pt | -0.403977 | 0.134659 |
| Total Δscore | 17452400 | -17452400 |
| Avg game Δscore | 198.322727 | -66.107576 |
| Avg round Δscore | 18.591323 | -6.197108 |
|  |  |  |
| Win rate | 0.201396 | 0.211725 |
| Deal-in rate | 0.120611 | 0.118825 |
| Call rate | 0.252337 | 0.297869 |
| Riichi rate | 0.214896 | 0.181350 |
| Ryukyoku rate | 0.167898 | 0.167898 |
|  |  |  |
| Avg winning Δscore | 6895.280813 | 6456.797252 |
| Avg winning Δscore as dealer | 8896.540353 | 8411.978003 |
| Avg winning Δscore as non-dealer | 6126.663299 | 5708.054700 |
| Avg riichi winning Δscore | 8226.234445 | 8055.765599 |
| Avg open winning Δscore | 5268.198657 | 4950.033985 |
| Avg dama winning Δscore | 6817.069924 | 6493.118161 |
| Avg ryukyoku Δscore | 33.715707 | -11.238569 |
|  |  |  |
| Avg winning turn | 11.181971 | 11.112885 |
| Avg riichi winning turn | 11.227615 | 11.129407 |
| Avg open winning turn | 11.090254 | 11.056002 |
| Avg dama winning turn | 11.307411 | 11.238720 |
|  |  |  |
| Avg deal-in turn | 11.509212 | 11.491703 |
| Avg deal-in Δscore | -5268.248220 | -5371.322657 |
| Avg deal-in Δscore to dealer | -6986.856002 | -7088.977424 |
| Avg deal-in Δscore to non-dealer | -4661.518881 | -4764.608616 |
|  |  |  |
| Chasing riichi rate | 0.183338 | 0.173439 |
| Riichi chased rate | 0.167614 | 0.188733 |
| Winning rate after riichi | 0.460896 | 0.482050 |
| Deal-in rate after riichi | 0.157304 | 0.149767 |
| Avg riichi turn | 7.970679 | 7.798867 |
| Avg riichi Δscore | 3023.188801 | 3154.216686 |
|  |  |  |
| Avg number of calls | 1.441297 | 1.447179 |
| Winning rate after call | 0.316803 | 0.313935 |
| Deal-in rate after call | 0.139772 | 0.135112 |
| Avg call Δscore | 997.837715 | 861.855676 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.225344 | 0.233891 |
| Dealer wins/all wins | 0.277492 | 0.276910 |
| Deal-in to dealer/all deal-ins | 0.260921 | 0.261023 |
|  |  |  |
| Yakuman (rate) | 155 (0.000165115) | 510 (0.000181094) |
| Nagashi mangan (rate) | 29 (0.000030893) | 94 (0.000033378) |

</details>

Swapping Challenger and Champion.

<details>

|  | mortal1-b40c192-t22040618 (x1) | mortal2-b75c256-t22100115 (x3) |
|---:|:---|:---|
| Games | 184000 | 552000 |
| Rounds | 1949267 | 5847801 |
| Rounds as dealer | 490808 | 1458459 |
|  |  |  |
| 1st (rate) | 43859 (0.238364) | 140141 (0.253879) |
| 2nd (rate) | 46785 (0.254266) | 137215 (0.248578) |
| 3rd (rate) | 49108 (0.266891) | 134892 (0.244370) |
| 4th (rate) | 44248 (0.240478) | 139752 (0.253174) |
| Tobi(rate) | 13951 (0.075821) | 42533 (0.077053) |
| Avg rank | 2.509484 | 2.496839 |
| Total pt | 79155 | -79155 |
| Avg game pt | 0.430190 | -0.143397 |
| Total Δscore | -27210700 | 27210700 |
| Avg game Δscore | -147.884239 | 49.294746 |
| Avg round Δscore | -13.959452 | 4.653151 |
|  |  |  |
| Win rate | 0.214427 | 0.203815 |
| Deal-in rate | 0.114537 | 0.116552 |
| Call rate | 0.298879 | 0.252706 |
| Riichi rate | 0.182324 | 0.216687 |
| Ryukyoku rate | 0.178674 | 0.178674 |
|  |  |  |
| Avg winning Δscore | 6536.850290 | 6979.860891 |
| Avg winning Δscore as dealer | 8484.620894 | 8965.835300 |
| Avg winning Δscore as non-dealer | 5789.726810 | 6209.649582 |
| Avg riichi winning Δscore | 8147.536626 | 8309.278377 |
| Avg open winning Δscore | 5020.329473 | 5337.498028 |
| Avg dama winning Δscore | 6549.632467 | 6890.664374 |
| Avg ryukyoku Δscore | -27.239265 | 9.079755 |
|  |  |  |
| Avg winning turn | 11.155595 | 11.208098 |
| Avg riichi winning turn | 11.160617 | 11.252630 |
| Avg open winning turn | 11.102974 | 11.112422 |
| Avg dama winning turn | 11.299210 | 11.347049 |
|  |  |  |
| Avg deal-in turn | 11.557105 | 11.590087 |
| Avg deal-in Δscore | -5644.898169 | -5536.106677 |
| Avg deal-in Δscore to dealer | -7381.606863 | -7276.555470 |
| Avg deal-in Δscore to non-dealer | -5026.326180 | -4916.529358 |
|  |  |  |
| Chasing riichi rate | 0.190536 | 0.200899 |
| Riichi chased rate | 0.225179 | 0.203861 |
| Winning rate after riichi | 0.485906 | 0.464404 |
| Deal-in rate after riichi | 0.147902 | 0.154747 |
| Avg riichi turn | 7.823572 | 8.000077 |
| Avg riichi Δscore | 3201.643510 | 3069.475433 |
|  |  |  |
| Avg number of calls | 1.445138 | 1.438486 |
| Winning rate after call | 0.315708 | 0.317395 |
| Deal-in rate after call | 0.129363 | 0.135622 |
| Avg call Δscore | 892.875497 | 1007.683110 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.236096 | 0.228368 |
| Dealer wins/all wins | 0.277237 | 0.279448 |
| Deal-in to dealer/all deal-ins | 0.262632 | 0.262530 |
|  |  |  |
| Yakuman (rate) | 317 (0.000162625) | 913 (0.000156127) |
| Nagashi mangan (rate) | 57 (0.000029242) | 203 (0.000034714) |

</details>

### mortal2-b75c256-t22092920 and mortal2-b75c256-t22100115
<details>

|  | mortal2-b75c256-t22100115 (x1) | mortal2-b75c256-t22092920 (x3) |
|---:|:---|:---|
| Games | 186000 | 558000 |
| Rounds | 2000904 | 6002712 |
| Rounds as dealer | 502601 | 1498303 |
|  |  |  |
| 1st (rate) | 49051 (0.263715) | 136949 (0.245428) |
| 2nd (rate) | 45866 (0.246591) | 140134 (0.251136) |
| 3rd (rate) | 43639 (0.234618) | 142361 (0.255127) |
| 4th (rate) | 47444 (0.255075) | 138556 (0.248308) |
| Tobi(rate) | 14221 (0.076457) | 39735 (0.071210) |
| Avg rank | 2.481054 | 2.506315 |
| Total pt | 73620 | -73620 |
| Avg game pt | 0.395806 | -0.131935 |
| Total Δscore | 52811900 | -52811900 |
| Avg game Δscore | 283.934946 | -94.644982 |
| Avg round Δscore | 26.394020 | -8.798007 |
|  |  |  |
| Win rate | 0.203744 | 0.202917 |
| Deal-in rate | 0.117473 | 0.109930 |
| Call rate | 0.250955 | 0.288268 |
| Riichi rate | 0.221043 | 0.177158 |
| Ryukyoku rate | 0.191611 | 0.191611 |
|  |  |  |
| Avg winning Δscore | 6968.286269 | 6705.932998 |
| Avg winning Δscore as dealer | 8960.625295 | 8626.184921 |
| Avg winning Δscore as non-dealer | 6189.807733 | 5950.988887 |
| Avg riichi winning Δscore | 8293.544525 | 8325.528002 |
| Avg open winning Δscore | 5273.455000 | 5181.309799 |
| Avg dama winning Δscore | 6928.691741 | 6573.246859 |
| Avg ryukyoku Δscore | 96.472055 | -32.157352 |
|  |  |  |
| Avg winning turn | 11.311866 | 11.193509 |
| Avg riichi winning turn | 11.339468 | 11.191932 |
| Avg open winning turn | 11.237098 | 11.142963 |
| Avg dama winning turn | 11.442988 | 11.348828 |
|  |  |  |
| Avg deal-in turn | 11.590463 | 11.601442 |
| Avg deal-in Δscore | -5483.272283 | -5512.258725 |
| Avg deal-in Δscore to dealer | -7189.870072 | -7213.092520 |
| Avg deal-in Δscore to non-dealer | -4861.026769 | -4901.751503 |
|  |  |  |
| Chasing riichi rate | 0.178247 | 0.167947 |
| Riichi chased rate | 0.158336 | 0.185398 |
| Winning rate after riichi | 0.458852 | 0.481619 |
| Deal-in rate after riichi | 0.152395 | 0.145284 |
| Avg riichi turn | 8.068444 | 7.821574 |
| Avg riichi Δscore | 3062.570825 | 3297.736095 |
|  |  |  |
| Avg number of calls | 1.425741 | 1.457532 |
| Winning rate after call | 0.313837 | 0.305507 |
| Deal-in rate after call | 0.135666 | 0.129938 |
| Avg call Δscore | 1003.224226 | 928.448665 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.227891 | 0.229417 |
| Dealer wins/all wins | 0.280956 | 0.282201 |
| Deal-in to dealer/all deal-ins | 0.267191 | 0.264136 |
|  |  |  |
| Yakuman (rate) | 357 (0.000178419) | 898 (0.000149599) |
| Nagashi mangan (rate) | 83 (0.000041481) | 157 (0.000026155) |

</details>

Swapping Challenger and Champion.

<details>

|  | mortal2-b75c256-t22092920 (x1) | mortal2-b75c256-t22100115 (x3) |
|---:|:---|:---|
| Games | 468000 | 1404000 |
| Rounds | 4970641 | 14911923 |
| Rounds as dealer | 1238715 | 3731926 |
|  |  |  |
| 1st (rate) | 111524 (0.238299) | 356476 (0.253900) |
| 2nd (rate) | 118662 (0.253551) | 349338 (0.248816) |
| 3rd (rate) | 124344 (0.265692) | 343656 (0.244769) |
| 4th (rate) | 113470 (0.242457) | 354530 (0.252514) |
| Tobi(rate) | 34046 (0.072748) | 111105 (0.079135) |
| Avg rank | 2.512308 | 2.495897 |
| Total pt | 58500 | -58500 |
| Avg game pt | 0.125000 | -0.041667 |
| Total Δscore | -87196300 | 87196300 |
| Avg game Δscore | -186.316880 | 62.105627 |
| Avg round Δscore | -17.542265 | 5.847422 |
|  |  |  |
| Win rate | 0.204451 | 0.204526 |
| Deal-in rate | 0.107986 | 0.115582 |
| Call rate | 0.289977 | 0.252561 |
| Riichi rate | 0.175660 | 0.218852 |
| Ryukyoku rate | 0.186354 | 0.186354 |
|  |  |  |
| Avg winning Δscore | 6743.228986 | 6999.379451 |
| Avg winning Δscore as dealer | 8653.390068 | 8978.546069 |
| Avg winning Δscore as non-dealer | 5997.652908 | 6231.775811 |
| Avg riichi winning Δscore | 8362.778335 | 8330.640680 |
| Avg open winning Δscore | 5249.764725 | 5334.895698 |
| Avg dama winning Δscore | 6589.944050 | 6939.807426 |
| Avg ryukyoku Δscore | -89.135558 | 29.711853 |
|  |  |  |
| Avg winning turn | 11.133417 | 11.256085 |
| Avg riichi winning turn | 11.148786 | 11.294117 |
| Avg open winning turn | 11.070347 | 11.167886 |
| Avg dama winning turn | 11.281376 | 11.394572 |
|  |  |  |
| Avg deal-in turn | 11.628012 | 11.622673 |
| Avg deal-in Δscore | -5636.075222 | -5611.253453 |
| Avg deal-in Δscore to dealer | -7372.437410 | -7336.220817 |
| Avg deal-in Δscore to non-dealer | -5024.106218 | -4989.261629 |
|  |  |  |
| Chasing riichi rate | 0.187202 | 0.200789 |
| Riichi chased rate | 0.228946 | 0.202108 |
| Winning rate after riichi | 0.486471 | 0.462388 |
| Deal-in rate after riichi | 0.146565 | 0.153587 |
| Avg riichi turn | 7.799500 | 8.029891 |
| Avg riichi Δscore | 3316.085242 | 3065.027878 |
|  |  |  |
| Avg number of calls | 1.465461 | 1.434810 |
| Winning rate after call | 0.309184 | 0.317169 |
| Deal-in rate after call | 0.128032 | 0.134245 |
| Avg call Δscore | 948.564180 | 1011.617646 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.230323 | 0.228383 |
| Dealer wins/all wins | 0.280742 | 0.279457 |
| Deal-in to dealer/all deal-ins | 0.260597 | 0.265020 |
|  |  |  |
| Yakuman (rate) | 731 (0.000147064) | 2437 (0.000163426) |
| Nagashi mangan (rate) | 137 (0.000027562) | 575 (0.000038560) |

</details>

### mortal3-b24c512-t22122709 and mortal1-b40c192-t22040618
<details>

|  | mortal3-b24c512-t22122709 (x1) | mortal1-b40c192-t22040618 (x3) |
|---:|:---|:---|
| Games | 1002000 | 3006000 |
| Rounds | 10709438 | 32128314 |
| Rounds as dealer | 2662468 | 8046970 |
|  |  |  |
| 1st (rate) | 252727 (0.252223) | 749273 (0.249259) |
| 2nd (rate) | 248648 (0.248152) | 753352 (0.250616) |
| 3rd (rate) | 250132 (0.249633) | 751868 (0.250122) |
| 4th (rate) | 250493 (0.249993) | 751507 (0.250002) |
| Tobi(rate) | 66970 (0.066836) | 198442 (0.066015) |
| Avg rank | 2.497396 | 2.500868 |
| Total pt | 118035 | -118035 |
| Avg game pt | 0.117799 | -0.039266 |
| Total Δscore | -63363900 | 63363900 |
| Avg game Δscore | -63.237425 | 21.079142 |
| Avg round Δscore | -5.916641 | 1.972214 |
|  |  |  |
| Win rate | 0.208219 | 0.210348 |
| Deal-in rate | 0.119234 | 0.119914 |
| Call rate | 0.294273 | 0.295517 |
| Riichi rate | 0.188882 | 0.181314 |
| Ryukyoku rate | 0.165333 | 0.165333 |
|  |  |  |
| Avg winning Δscore | 6445.727072 | 6429.191949 |
| Avg winning Δscore as dealer | 8399.590822 | 8368.224417 |
| Avg winning Δscore as non-dealer | 5717.250013 | 5694.814795 |
| Avg riichi winning Δscore | 8021.969607 | 8016.244281 |
| Avg open winning Δscore | 4828.263260 | 4928.192270 |
| Avg dama winning Δscore | 6699.031669 | 6438.613051 |
| Avg ryukyoku Δscore | -9.543246 | 3.181082 |
|  |  |  |
| Avg winning turn | 11.112528 | 11.089737 |
| Avg riichi winning turn | 11.135377 | 11.105556 |
| Avg open winning turn | 11.076537 | 11.029325 |
| Avg dama winning turn | 11.162149 | 11.228365 |
|  |  |  |
| Avg deal-in turn | 11.403147 | 11.468034 |
| Avg deal-in Δscore | -5224.409601 | -5263.605252 |
| Avg deal-in Δscore to dealer | -6915.714079 | -6979.817903 |
| Avg deal-in Δscore to non-dealer | -4632.352717 | -4665.443922 |
|  |  |  |
| Chasing riichi rate | 0.162380 | 0.168922 |
| Riichi chased rate | 0.173606 | 0.172392 |
| Winning rate after riichi | 0.480132 | 0.481958 |
| Deal-in rate after riichi | 0.150001 | 0.150695 |
| Avg riichi turn | 7.807936 | 7.789643 |
| Avg riichi Δscore | 3133.117891 | 3146.475994 |
|  |  |  |
| Avg number of calls | 1.432674 | 1.449256 |
| Winning rate after call | 0.313738 | 0.313302 |
| Deal-in rate after call | 0.133888 | 0.137132 |
| Avg call Δscore | 851.132637 | 860.015782 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.227460 | 0.230700 |
| Dealer wins/all wins | 0.271583 | 0.274697 |
| Deal-in to dealer/all deal-ins | 0.259292 | 0.258455 |
|  |  |  |
| Yakuman (rate) | 1369 (0.000127831) | 4884 (0.000152015) |
| Nagashi mangan (rate) | 770 (0.000071899) | 932 (0.000029009) |

</details>

### mortal3-b24c512-t22122709 and mortal2-b75c256-t22092920
<details>

|  | mortal3-b24c512-t22122709 (x1) | mortal2-b75c256-t22092920 (x3) |
|---:|:---|:---|
| Games | 368000 | 1104000 |
| Rounds | 3974175 | 11922525 |
| Rounds as dealer | 1000068 | 2974107 |
|  |  |  |
| 1st (rate) | 93704 (0.254630) | 274296 (0.248457) |
| 2nd (rate) | 91424 (0.248435) | 276576 (0.250522) |
| 3rd (rate) | 92006 (0.250016) | 275994 (0.249995) |
| 4th (rate) | 90866 (0.246918) | 277134 (0.251027) |
| Tobi(rate) | 26211 (0.071226) | 73783 (0.066832) |
| Avg rank | 2.489223 | 2.503592 |
| Total pt | 280530 | -280530 |
| Avg game pt | 0.762310 | -0.254103 |
| Total Δscore | 16622600 | -16622600 |
| Avg game Δscore | 45.170109 | -15.056703 |
| Avg round Δscore | 4.182654 | -1.394218 |
|  |  |  |
| Win rate | 0.210877 | 0.201749 |
| Deal-in rate | 0.116472 | 0.111503 |
| Call rate | 0.293698 | 0.286804 |
| Riichi rate | 0.193439 | 0.176834 |
| Ryukyoku rate | 0.187923 | 0.187923 |
|  |  |  |
| Avg winning Δscore | 6506.988743 | 6671.600717 |
| Avg winning Δscore as dealer | 8465.651080 | 8586.955954 |
| Avg winning Δscore as non-dealer | 5762.116967 | 5925.650595 |
| Avg riichi winning Δscore | 8093.180291 | 8287.979161 |
| Avg open winning Δscore | 4825.511770 | 5152.033307 |
| Avg dama winning Δscore | 6772.418571 | 6511.443291 |
| Avg ryukyoku Δscore | 60.018880 | -20.006293 |
|  |  |  |
| Avg winning turn | 11.258288 | 11.165158 |
| Avg riichi winning turn | 11.253880 | 11.170332 |
| Avg open winning turn | 11.244792 | 11.112081 |
| Avg dama winning turn | 11.319405 | 11.308801 |
|  |  |  |
| Avg deal-in turn | 11.490814 | 11.573875 |
| Avg deal-in Δscore | -5424.654554 | -5386.304929 |
| Avg deal-in Δscore to dealer | -7072.028040 | -7092.616967 |
| Avg deal-in Δscore to non-dealer | -4829.650323 | -4785.963319 |
|  |  |  |
| Chasing riichi rate | 0.156698 | 0.162294 |
| Riichi chased rate | 0.163280 | 0.167319 |
| Winning rate after riichi | 0.478186 | 0.481160 |
| Deal-in rate after riichi | 0.144392 | 0.145621 |
| Avg riichi turn | 7.893116 | 7.806837 |
| Avg riichi Δscore | 3179.229826 | 3296.884365 |
|  |  |  |
| Avg number of calls | 1.419072 | 1.460339 |
| Winning rate after call | 0.311547 | 0.304823 |
| Deal-in rate after call | 0.130112 | 0.132365 |
| Avg call Δscore | 856.935365 | 918.769254 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.230885 | 0.226693 |
| Dealer wins/all wins | 0.275518 | 0.280295 |
| Deal-in to dealer/all deal-ins | 0.265345 | 0.260265 |
|  |  |  |
| Yakuman (rate) | 486 (0.000122290) | 1736 (0.000145607) |
| Nagashi mangan (rate) | 344 (0.000086559) | 268 (0.000022478) |

</details>

### mortal3-b24c512-t22122709 and mortal2-b75c256-t22100115
<details>

|  | mortal3-b24c512-t22122709 (x1) | mortal2-b75c256-t22100115 (x3) |
|---:|:---|:---|
| Games | 386000 | 1158000 |
| Rounds | 4083546 | 12250638 |
| Rounds as dealer | 1022028 | 3061518 |
|  |  |  |
| 1st (rate) | 93099 (0.241189) | 292901 (0.252937) |
| 2nd (rate) | 97284 (0.252031) | 288716 (0.249323) |
| 3rd (rate) | 102923 (0.266640) | 283077 (0.244453) |
| 4th (rate) | 92694 (0.240140) | 293306 (0.253287) |
| Tobi(rate) | 29413 (0.076199) | 89107 (0.076949) |
| Avg rank | 2.505731 | 2.498090 |
| Total pt | 243000 | -243000 |
| Avg game pt | 0.629534 | -0.209845 |
| Total Δscore | -79207200 | 79207200 |
| Avg game Δscore | -205.200000 | 68.400000 |
| Avg round Δscore | -19.396671 | 6.465557 |
|  |  |  |
| Win rate | 0.212155 | 0.204081 |
| Deal-in rate | 0.113413 | 0.116293 |
| Call rate | 0.295992 | 0.252329 |
| Riichi rate | 0.190717 | 0.217100 |
| Ryukyoku rate | 0.180088 | 0.180088 |
|  |  |  |
| Avg winning Δscore | 6566.458089 | 6987.499520 |
| Avg winning Δscore as dealer | 8535.578228 | 8976.920737 |
| Avg winning Δscore as non-dealer | 5823.653642 | 6218.547096 |
| Avg riichi winning Δscore | 8154.003066 | 8328.467883 |
| Avg open winning Δscore | 4922.635337 | 5334.771576 |
| Avg dama winning Δscore | 6854.794355 | 6896.380902 |
| Avg ryukyoku Δscore | -33.202474 | 11.067491 |
|  |  |  |
| Avg winning turn | 11.178651 | 11.212072 |
| Avg riichi winning turn | 11.189294 | 11.254969 |
| Avg open winning turn | 11.139738 | 11.113306 |
| Avg dama winning turn | 11.279740 | 11.369807 |
|  |  |  |
| Avg deal-in turn | 11.528364 | 11.617473 |
| Avg deal-in Δscore | -5605.944361 | -5549.620576 |
| Avg deal-in Δscore to dealer | -7342.659893 | -7304.122310 |
| Avg deal-in Δscore to non-dealer | -4988.584666 | -4928.993379 |
|  |  |  |
| Chasing riichi rate | 0.185512 | 0.205671 |
| Riichi chased rate | 0.230286 | 0.205672 |
| Winning rate after riichi | 0.483306 | 0.463756 |
| Deal-in rate after riichi | 0.147658 | 0.156078 |
| Avg riichi turn | 7.846740 | 8.008423 |
| Avg riichi Δscore | 3180.435567 | 3063.362113 |
|  |  |  |
| Avg number of calls | 1.430087 | 1.439802 |
| Winning rate after call | 0.316357 | 0.318721 |
| Deal-in rate after call | 0.126356 | 0.135419 |
| Avg call Δscore | 885.609639 | 1014.625726 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.232180 | 0.227651 |
| Dealer wins/all wins | 0.273903 | 0.278770 |
| Deal-in to dealer/all deal-ins | 0.262251 | 0.261303 |
|  |  |  |
| Yakuman (rate) | 512 (0.000125381) | 946 (0.000158849) |
| Nagashi mangan (rate) | 341 (0.000083506) | 25 (0.000034692) |

</details>

### mortal3-b24c512-t22122709 and mortal3-b24c512-t22121413
<details>

|  | mortal3-b24c512-t22122709 (x1) | mortal3-b24c512-t22121413 (x3) |
|---:|:---|:---|
| Games | 1000000 | 3000000 |
| Rounds | 10675990 | 32027970 |
| Rounds as dealer | 2640435 | 8035555 |
|  |  |  |
| 1st (rate) | 256715 (0.256715) | 743285 (0.247762) |
| 2nd (rate) | 253315 (0.253315) | 746685 (0.248895) |
| 3rd (rate) | 246115 (0.246115) | 753885 (0.251295) |
| 4th (rate) | 243855 (0.243855) | 756145 (0.252048) |
| Tobi(rate) | 62001 (0.062001) | 194826 (0.064942) |
| Avg rank | 2.477110 | 2.507630 |
| Total pt | 1583100 | -1583100 |
| Avg game pt | 1.583100 | -0.527700 |
| Total Δscore | 146170800 | -146170800 |
| Avg game Δscore | 146.170800 | -48.723600 |
| Avg round Δscore | 13.691545 | -4.563848 |
|  |  |  |
| Win rate | 0.206694 | 0.213079 |
| Deal-in rate | 0.116511 | 0.125168 |
| Call rate | 0.294321 | 0.303311 |
| Riichi rate | 0.185257 | 0.177704 |
| Ryukyoku rate | 0.159419 | 0.159419 |
|  |  |  |
| Avg winning Δscore | 6404.250681 | 6286.824125 |
| Avg winning Δscore as dealer | 8361.741949 | 8224.710855 |
| Avg winning Δscore as non-dealer | 5676.332225 | 5551.529875 |
| Avg riichi winning Δscore | 8047.830008 | 7978.528810 |
| Avg open winning Δscore | 4777.098429 | 4775.399159 |
| Avg dama winning Δscore | 6644.025031 | 6379.190680 |
| Avg ryukyoku Δscore | -18.918538 | 6.306179 |
|  |  |  |
| Avg winning turn | 10.978592 | 10.967597 |
| Avg riichi winning turn | 11.051985 | 11.045908 |
| Avg open winning turn | 10.885215 | 10.873950 |
| Avg dama winning turn | 11.066078 | 11.042189 |
|  |  |  |
| Avg deal-in turn | 11.295449 | 11.278288 |
| Avg deal-in Δscore | -5139.674387 | -5124.328080 |
| Avg deal-in Δscore to dealer | -6859.246117 | -6821.157102 |
| Avg deal-in Δscore to non-dealer | -4538.744279 | -4533.356136 |
|  |  |  |
| Chasing riichi rate | 0.163170 | 0.171444 |
| Riichi chased rate | 0.177894 | 0.175603 |
| Winning rate after riichi | 0.477664 | 0.474371 |
| Deal-in rate after riichi | 0.148014 | 0.151642 |
| Avg riichi turn | 7.744334 | 7.742333 |
| Avg riichi Δscore | 3157.026400 | 3077.846775 |
|  |  |  |
| Avg number of calls | 1.441144 | 1.454572 |
| Winning rate after call | 0.316271 | 0.317613 |
| Deal-in rate after call | 0.130817 | 0.139817 |
| Avg call Δscore | 863.542555 | 834.038992 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.226533 | 0.233608 |
| Dealer wins/all wins | 0.271064 | 0.275063 |
| Deal-in to dealer/all deal-ins | 0.258966 | 0.258314 |
|  |  |  |
| Yakuman (rate) | 1273 (0.000119240) | 6058 (0.000189147) |
| Nagashi mangan (rate) | 774 (0.000072499) | 2320 (0.000072437) |

</details>

Swapping Challenger and Champion.

<details>

|  | mortal3-b24c512-t22121413 (x1) | mortal3-b24c512-t22122709 (x3) |
|---:|:---|:---|
| Games | 1000000 | 3000000 |
| Rounds | 10671322 | 32013966 |
| Rounds as dealer | 2697993 | 7973329 |
|  |  |  |
| 1st (rate) | 243160 (0.243160) | 756840 (0.252280) |
| 2nd (rate) | 248268 (0.248268) | 751732 (0.250577) |
| 3rd (rate) | 253542 (0.253542) | 746458 (0.248819) |
| 4th (rate) | 255030 (0.255030) | 744970 (0.248323) |
| Tobi(rate) | 66964 (0.066964) | 192123 (0.064041) |
| Avg rank | 2.520442 | 2.493186 |
| Total pt | -1372590 | 1372590 |
| Avg game pt | -1.372590 | 0.457530 |
| Total Δscore | -129515800 | 129515800 |
| Avg game Δscore | -129.515800 | 43.171933 |
| Avg round Δscore | -12.136809 | 4.045603 |
|  |  |  |
| Win rate | 0.213817 | 0.208091 |
| Deal-in rate | 0.125421 | 0.117515 |
| Call rate | 0.301421 | 0.293180 |
| Riichi rate | 0.180298 | 0.187937 |
| Ryukyoku rate | 0.166519 | 0.166519 |
|  |  |  |
| Avg winning Δscore | 6339.122361 | 6446.471119 |
| Avg winning Δscore as dealer | 8298.133576 | 8413.219744 |
| Avg winning Δscore as non-dealer | 5590.723615 | 5713.335004 |
| Avg riichi winning Δscore | 7999.275659 | 8059.222245 |
| Avg open winning Δscore | 4806.043945 | 4804.190094 |
| Avg dama winning Δscore | 6432.490775 | 6700.040250 |
| Avg ryukyoku Δscore | 23.501685 | -7.833895 |
|  |  |  |
| Avg winning turn | 11.052582 | 11.067857 |
| Avg riichi winning turn | 11.113263 | 11.117478 |
| Avg open winning turn | 10.970769 | 10.995953 |
| Avg dama winning turn | 11.130811 | 11.153572 |
|  |  |  |
| Avg deal-in turn | 11.369043 | 11.398171 |
| Avg deal-in Δscore | -5189.700710 | -5198.814820 |
| Avg deal-in Δscore to dealer | -6903.797579 | -6916.602151 |
| Avg deal-in Δscore to non-dealer | -4597.567461 | -4604.788905 |
|  |  |  |
| Chasing riichi rate | 0.177665 | 0.168433 |
| Riichi chased rate | 0.175032 | 0.177377 |
| Winning rate after riichi | 0.476806 | 0.479996 |
| Deal-in rate after riichi | 0.152919 | 0.149520 |
| Avg riichi turn | 7.784426 | 7.788808 |
| Avg riichi Δscore | 3090.200897 | 3165.365615 |
|  |  |  |
| Avg number of calls | 1.449837 | 1.437720 |
| Winning rate after call | 0.315467 | 0.315524 |
| Deal-in rate after call | 0.140190 | 0.132097 |
| Avg call Δscore | 832.163243 | 861.704796 |
|  |  |  |
| Dealer wins/all dealer rounds | 0.233776 | 0.226878 |
| Dealer wins/all wins | 0.276426 | 0.271543 |
| Deal-in to dealer/all deal-ins | 0.256754 | 0.256952 |
|  |  |  |
| Yakuman (rate) | 2065 (0.000193509) | 4069 (0.000127101) |
| Nagashi mangan (rate) | 769 (0.000072062) | 2417 (0.000075498) |

</details>

## Tenhou
Tenhou does not allow AI to play in ranked lobby without their permission, therefore I only compared how close Mortal's decisions matched those of other verified AIs that had previously played in Tenhou tokujou. I also sampled some tokujou and houou games to check against Mortal.

The model used in the statistic experiment was mortal1-b40c192-t22040618.

[![](../assets/match-rate.png)](../assets/match-rate.png)
