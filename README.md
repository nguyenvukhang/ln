# `ln`

An opinionated `git log` wrapper in 200 lines of C.

> Effectively passes extra args to `git log` and does some aesthetic processing
> on its output.  
> Uses `less` as the pager, if available.

#### Usage

1. Compile `main.c`
2. Rename it to `git-ln` and put it somewhere in your `PATH`.
3. Enjoy using `git ln` in the place of `git log`.

```
git ln -n 15 --all
```
![git-ln](https://github.com/user-attachments/assets/c1e10bf0-eb37-4376-8e5d-a5890fe66459)
