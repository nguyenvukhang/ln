#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define OUTPUT stdout
#define print(text, len) fwrite(text, sizeof(char), len, OUTPUT)
#define Print(text) print(text, sizeof(text) - 1)
#define TTY_Print(t, f) IS_ATTY ? Print(t) : Print(f)
#define SP ";-;_"
#define SP_L (sizeof(SP) - 1)
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

#define CLOSE_DUP(c, d, f) \
  close(c);                \
  dup2(d, f);

#define RUN_GIT_LOG                                    \
  char buf[BUF_SIZE];                                  \
  for (; fgets(buf, BUF_SIZE, stdin);) send_line(buf); \
  return 0;

static int t;  // time since unix epoch (in secs)
static int IS_ATTY;

void print_refs(char *x, int len) {
  TTY_Print(DARK_GRAY "{", "{");
  for (int i = 0; i < len; i++, x++) {
    if (strncmp(x, "origin", 6) == 0) {
      i += 6;
      x += 6;
      Print("*");
      continue;
    }
    if (strncmp(x, "\e[33m", 5) == 0) x[3] = '7';  // yellow -> gray.
    print(x, 1);
  }
  TTY_Print(DARK_GRAY "} ", "} ");
}

int print_time(const char *time) {
  int n = t - strtol(time, NULL, 10);
#define send(m, l)                              \
  if (n < m) return fprintf(OUTPUT, "%d" l, n); \
  else n /= m;
  send(60, "s") send(60, "m") send(24, "h") send(7, "d");
#undef send
  return fprintf(OUTPUT, "%dw", n);
}

int send_line(const char *buf) {
  static char *hash, *refs, *comment, *time;

  if (!(hash = strstr(buf, SP))) return print(buf, strlen(buf));

  refs = strstr((hash += SP_L), SP) + SP_L;
  comment = strstr(refs, SP) + SP_L;
  time = strstr(comment, SP) + SP_L;

  print(buf, hash - buf - SP_L);  // graph visual provided by `--graph`
  if (IS_ATTY) Print("\e[33m");
  print(hash, refs - hash - SP_L);
  Print(" ");

  int refs_l = comment - refs - SP_L;
  if (refs_l > 2 + 6 * IS_ATTY)
    print_refs(refs + 8 * IS_ATTY, refs_l - 16 * IS_ATTY);
  if (IS_ATTY) Print(RESET);

  print(comment, time - comment - SP_L);
  TTY_Print(DARK_GRAY " (" LIGHT_GRAY, " (");
  print_time(time);
  return TTY_Print(DARK_GRAY ")" RESET "\n", ")\n");
}

int main(const int argc, const char **argv) {
  t = time(NULL);
  IS_ATTY = isatty(STDOUT_FILENO);
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
#define FMT_ARGS SP "%h" SP "%D" SP "%s" SP "%at"
#define FMT_ARGS_COLORED SP "%h" SP "%C(auto)%D" SP "%s" SP "%at"

  // Start a fork for `git` where it writes to `p[1]`.
  if (p_git == 0) {
    const char *args[argc + 8];
    int i = 0;
#define ARG(v) args[i++] = v
    ARG("git");
    ARG("log");
    for (int j = 1; j < argc; ARG(argv[j++]));
    if (IS_ATTY) ARG("--color=always");
    IS_ATTY ? (ARG("--format=" FMT_ARGS_COLORED)) : (ARG("--format=" FMT_ARGS));
    ARG("--graph");
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
