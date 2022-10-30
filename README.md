## bookmark

Bookmark is a tool to gather information on a process' physical memory pages. This can be useful to understand the state of physical pages, whether they are mapped or not, and if they are in memory or swapped out.

Note that root is needed to read `/proc/<pid>/pagemap`

### Usage


#### Show page statistics grouped by backing file

```shell
$ sudo ./target/release/bookmark --pid $$ stats [--json]
[vdso] PageStats { swapped: 0, present: 1, unmapped: 1, total: 2 }
/usr/lib64/ld-2.32.so PageStats { swapped: 0, present: 46, unmapped: 0, total: 46 }
/usr/lib64/zsh/5.8/zsh/parameter.so PageStats { swapped: 0, present: 12, unmapped: 1, total: 13 }
/usr/lib64/libtinfo.so.6.2 PageStats { swapped: 0, present: 45, unmapped: 2, total: 47 }
/usr/lib64/zsh/5.8/zsh/datetime.so PageStats { swapped: 0, present: 6, unmapped: 0, total: 6 }
/usr/lib/locale/locale-archive PageStats { swapped: 0, present: 144, unmapped: 54432, total: 54576 }
/usr/lib64/libm-2.32.so PageStats { swapped: 0, present: 95, unmapped: 231, total: 326 }
/usr/lib64/libdl-2.32.so PageStats { swapped: 0, present: 5, unmapped: 1, total: 6 }
anon PageStats { swapped: 0, present: 26, unmapped: 8, total: 34 }
[vvar] PageStats { swapped: 0, present: 0, unmapped: 4, total: 4 }
/usr/lib64/zsh/5.8/zsh/complete.so PageStats { swapped: 0, present: 41, unmapped: 0, total: 41 }
/usr/bin/zsh PageStats { swapped: 0, present: 233, unmapped: 10, total: 243 }
/usr/lib64/gconv/gconv-modules.cache PageStats { swapped: 0, present: 7, unmapped: 0, total: 7 }
/usr/lib64/zsh/5.8/zsh/computil.so PageStats { swapped: 0, present: 20, unmapped: 0, total: 20 }
/usr/lib64/zsh/5.8/zsh/terminfo.so PageStats { swapped: 0, present: 5, unmapped: 0, total: 5 }
/usr/lib64/libnss_sss.so.2 PageStats { swapped: 0, present: 13, unmapped: 0, total: 13 }
[vsyscall] PageStats { swapped: 0, present: 0, unmapped: 1, total: 1 }
/usr/lib64/zsh/5.8/zsh/zutil.so PageStats { swapped: 0, present: 10, unmapped: 1, total: 11 }
/var/lib/sss/mc/passwd PageStats { swapped: 0, present: 32, unmapped: 2228, total: 2260 }
/usr/lib64/zsh/5.8/zsh/langinfo.so PageStats { swapped: 0, present: 5, unmapped: 0, total: 5 }
/usr/lib64/zsh/5.8/zsh/zle.so PageStats { swapped: 0, present: 86, unmapped: 0, total: 86 }
/usr/lib64/zsh/5.8/zsh/complist.so PageStats { swapped: 0, present: 18, unmapped: 1, total: 19 }
[heap] PageStats { swapped: 0, present: 652, unmapped: 10, total: 662 }
/usr/lib64/zsh/5.8/zsh/stat.so PageStats { swapped: 0, present: 6, unmapped: 0, total: 6 }
[stack] PageStats { swapped: 0, present: 94, unmapped: 0, total: 94 }
/usr/lib64/libc-2.32.so PageStats { swapped: 0, present: 398, unmapped: 57, total: 455 }
```

#### Show information per physical page

```shell
$ sudo ./target/release/bookmark --pid $$ list
7f02795c6000 bcf63c false Some("/usr/lib64/ld-linux-x86-64.so.2")
7f02795c7000 ba697b false Some("/usr/lib64/ld-linux-x86-64.so.2")
7fffdee91000 0 false Some("[stack]")
7fffdee92000 0 false Some("[stack]")
7fffdee93000 0 false Some("[stack]")
7fffdeeaf000 8025aa false Some("[stack]")
7fffdeeb0000 93acb0 false Some("[stack]")
7fffdeeb1000 b869dc false Some("[stack]")
[...]
```


### Tests

The code isn't very modular or clean, but there are some tests that can be run with

```shell
$ sudo cargo test
```
