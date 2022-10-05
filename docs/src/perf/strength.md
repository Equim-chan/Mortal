# Strength
## Simulation environment
The simulation uses a duplicate mahjong setting as follows

| Table | East start (0) | South start (1) | West start (2) | North start (3) |
|:---:|:---:|:---:|:---:|:---:|
| 1 | Challenger | Champion | Champion | Champion |
| 2 | Champion | Challenger | Champion | Champion |
| 3 | Champion | Champion | Challenger | Champion |
| 4 | Champion | Champion | Champion | Challenger |

In this setting, every 4 games are initialized with same random seed. The emulator guarantees that with the same (seed, kyoku, honba) tuple, the yama, haipai, dora/ura indicators and rinshan tiles are deterministic and reproducible.

The emulator is implemented in [`libriichi::arena`](https://github.com/Equim-chan/Mortal/tree/main/libriichi/src/arena).

## Mortal vs akochan
Challenger is akochan and Champion is Mortal.

### akochan with jun_pt = [90, 45, 0, -135]
|  | mortal1-b40c192-t22040618 (x3) | akochan (x1) |
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
|  | mortal1-b40c192-t22040618 (x3) | akochan (x1) |
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

## Mortal vs Mortal
### mortal2-b75c256-t22092920 and mortal1-b40c192-t22040618
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

Swapping Challenger and Champion.

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

### mortal2-b75c256-t22100115 and mortal1-b40c192-t22040618
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

Swapping Challenger and Champion.

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

### mortal2-b75c256-t22092920 and mortal2-b75c256-t22100115
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

Swapping Challenger and Champion.

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

## Tenhou
Tenhou does not allow AI to play in ranked lobby without their permission, therefore I only compared how closely Mortal's decisions matched those of other verified AIs that had previously played in Tenhou tokujou. I also sampled some tokujou and houou games to check against Mortal.

The model used in the figure is mortal1-b40c192-t22040618.

[![](../assets/match-rate.png)](../assets/match-rate.png)
