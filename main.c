#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define print(text, len) fwrite(text, sizeof(char), len, stdout)
#define Print(text) print(text, sizeof(text) - 1)
#define SEP ",.;:"
#define SEP_L (sizeof(SEP) - 1)
#define BUF_SIZE 0x200

#define RESET "\e[m"
#define DARK_GRAY "\e[38;5;240m"
#define LIGHT_GRAY "\e[38;5;246m"

#define READ 0
#define WRITE 1

#define INIT_PIPE(p)                 \
  if (pipe(p) < 0) {                 \
    perror("Unable to start pipe."); \
    return -1;                       \
  }

#define CLOSE_DUP(c, d, fno) \
  close(c);                  \
  dup2(d, fno);

#define RUN_GIT_LOG                                    \
  char buf[BUF_SIZE];                                  \
  for (; fgets(buf, BUF_SIZE, stdin);) send_line(buf); \
  return 0;

static int t;  // time since unix epoch (in secs)

void print_refs(char *x, int len) {
  Print(DARK_GRAY "{");
  for (int i = 0; i < len; i++, x++) {
    if (strncmp(x, "origin", 6) == 0) {
      i += 6;
      x += 6;
      Print("*");
      continue;
    }
    if (strncmp(x, "\e[33m", 5) == 0) {
      x[3] = '7';  // yellow -> gray.
    }
    print(x, 1);
  }
  Print(DARK_GRAY "}");
}

int print_time(const char *time) {
  int n = t - strtol(time, NULL, 10);
  static char buf[8];
#define send(l)                \
  snprintf(buf, 8, "%d" l, n); \
  return print(buf, strlen(buf));
  // clang-format off
    if  (n        < 60) { send("s"); }
    if ((n /= 60) < 60) { send("m"); }
    if ((n /= 60) < 24) { send("h"); }
    if ((n /= 24) < 7)  { send("d"); }
         n /=  7;         send("w");
  // clang-format on
#undef send
}

int send_line(const char *buf) {
  static char *hash, *refs, *comment, *time;

  if (!(hash = strstr(buf, SEP))) return print(buf, strlen(buf));

  refs = strstr((hash += SEP_L), SEP) + SEP_L;
  comment = strstr(refs, SEP) + SEP_L;
  time = strstr(comment, SEP) + SEP_L;

  print(buf, hash - buf - SEP_L);  // graph visual provided by `--graph`
  Print("\e[33m");
  print(hash, refs - hash - SEP_L);
  Print(" ");

  int refs_l = comment - refs - SEP_L;
  if (refs_l > 8) {
    print_refs(refs + 8, refs_l - 16);
    Print(" ");
  }
  Print(RESET);

  print(comment, time - comment - SEP_L);
  Print(DARK_GRAY " (" LIGHT_GRAY);
  print_time(time);
  return Print(DARK_GRAY ")" RESET "\n");
}

int main(const int argc, const char **argv) {
  t = time(NULL);
  pid_t p_git = 0, p_writer = 0;

  int p[2], q[2];  // pipes [READ, WRITE]

  INIT_PIPE(p);
  p_git = fork();

  char has_less = !system("which less > /dev/null 2>&1");

  if (has_less && p_git > 0) {
    INIT_PIPE(q);
    p_writer = fork();
  }

  /* At this point we have at most 3 execution threads (p_git, p_writer):
       (1, 2) => the parent (to pipe to `less`)
       (1, 0) => the child to spawn a writer
       (0, 0) => the child to spawn `git log`
       */

  // Start a fork for `git` where it writes to `p[1]`.
  if (p_git == 0) {
    const char *args[argc + 8];
    int i = 0;
#define ARG(v) args[i++] = v
    ARG("git");
    ARG("log");
    for (int j = 1; j < argc; args[i++] = argv[j++]);
    ARG("--color=always");
    ARG("--graph");
    ARG("--format=" SEP "%h" SEP "%C(auto)%D" SEP "%s" SEP "%at");
    ARG(NULL);
#undef ARG
    CLOSE_DUP(p[READ], p[WRITE], STDOUT_FILENO)
    execvp("git", (char *const *)args);
  }

  CLOSE_DUP(p[WRITE], p[READ], STDIN_FILENO)

  // Process `git`'s output and write it to regular old stdout.
  if (!has_less) {
    RUN_GIT_LOG
  }

  // Process `git`'s output AND send it to `q` to pass to `less`.
  if (p_writer == 0) {
    CLOSE_DUP(q[READ], q[WRITE], STDOUT_FILENO)
    RUN_GIT_LOG
  }

  CLOSE_DUP(q[WRITE], q[READ], STDIN_FILENO)
  return execlp("less", "less", "-RF", NULL);
}
