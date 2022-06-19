# cpiotools
Tools for examining CPIOs.

```
$ cpio-dump ./scratch/old.cpio
ino	mode	uid	gid	nlink	mtime	bytes	devmaj	devmin	rdevmaj	rdevmin	trailer	hash	name
0	16749	0	1	5	1	0	0	0	0	0	false	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23
1	16749	0	1	2	1	0	0	0	0	0	false	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/bin
2	33133	0	1	1	1	1015592	0	0	0	0	false	8pzu9JFQXgMEbshAKIAL+vvt/480b5Nb7BrRe7isDqU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/bin/bash
3	33133	0	1	1	1	7048	0	0	0	0	false	PMhm52KbdndNhNCdQKfsPLaZmqFbhWo36G+C554pWeg=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/bin/bashbug
4	41471	0	1	1	1	4	0	0	0	0	false	N9KxLV2avCo2TvlEh2fuA5OOODwChBk0d9x2GPS3xsI=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/bin/sh
5	16749	0	1	3	1	0	0	0	0	0	false	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/lib
6	16749	0	1	2	1	0	0	0	0	0	false	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/lib/bash
7	33133	0	1	1	1	16176	0	0	0	0	false	qWTcKSUkbki5RAh/T5SQ66v12MGmrQ3zfawLBSO0YfU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/lib/bash/basename
8	33133	0	1	1	1	16120	0	0	0	0	false	Vz48JSFGqt/AbSUJN1Kha23ZV8NLd1/RZi2d4YGWTyE=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/lib/bash/dirname
```

Comparing two CPIOs:

```diff
$ diff -u <(cpio-dump ./left.cpio) <(cpio-dump ./right.cpio)
--- /dev/fd/63	2022-06-19 08:30:23.535480908 -0400
+++ /dev/fd/62	2022-06-19 08:30:23.533480868 -0400
@@ -145,7 +145,3 @@
 143	16749	0	1	2	1	0	0	0	0	0	false	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/share/locale/zh_TW/LC_MESSAGES
 144	33060	0	1	1	1	93961	0	0	0	0	false	WWhEzPnW3nPUUgiWF3J6FWI9cnvdCTWh2Gpe1GhFTUE=	nix/store/bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23/share/locale/zh_TW/LC_MESSAGES/bash.mo
 0	0	0	0	1	0	0	0	0	0	0	true	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	TRAILER!!!
--	-	-	-	-	-	68	-	-	-	-	-	_	skipped null bytes
-0	33188	0	1	1	1655641823	516	0	0	0	0	false	KD8P46VjytTnAqoAUB2hhxWK+x/UXpkZ7sKTod2jA10=	nix/.nix-netboot-serve-db/registration//bbp2qxxsihmx87w30qvi1c020vggkdqn-bash-interactive-4.4-p23
-0	0	0	0	1	0	0	0	0	0	0	true	47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=	TRAILER!!!
--	-	-	-	-	-	176	-	-	-	-	-	_	skipped null bytes
```
