# Meta
## What does the name "Mortal" mean?
Mortal in this context means something opposed to supernatural or immortal.

There is no superpower in mahjong, nor any supernatural "fate's bless" to be relied on. Everyone is "mortal" and the AI is no different.

What the AI does is merely believing exclusively in the grand truth and never become emotional, with enough learning and practice and that's it. I've always believed that one of the reasons why many human players cannot defeat AIs in mahjong is that they are prone to becoming emotional, leading to minor but often fatal mistakes, which sometimes the players themselves deny to admit and blame on luck instead.

I got the inspiration from Kajiki Yumi, a character from Saki. She is indeed a mortal compared with other opponents she has played against, yet she tried her best as a mortal fighting for her own objective. "kajusan" was one of the names I have thought of, but I thought there were already a few mahjong projects based on character names from Saki.

I had a hard time thinking for a name. The project started with "OpenPhoenix", because it was at first a reproduction of Suphx, but after I changed many parts of it, it became less and less alike to Suphx, then I renamed it to "Reishanten". In the end, I thought the name was too hard to read and get its meaning, I came up with the name "Mortal".

## When was the project started?
The project started on 2021-04-22. A prototype of `PlayerState` was made that day.

## Why AGPL?
First of all, it is because the shanten algorithm ([`/libriichi/src/algo/shanten.rs`](https://github.com/Equim-chan/Mortal/tree/main/libriichi/src/algo/shanten.rs)) is a Rust port of [tomohxx/shanten-number-calculator](https://github.com/tomohxx/shanten-number-calculator), which is licensed under GPL. As for the reason for it to be **A**GPL, my consideration is this project is natually easy to be exploited, such as being used for cheats or getting renamed then sold to unaware people, so even if I can't _really_ stop such exploit, at least I want to do my part and make my attempt on what I can to stop this.
