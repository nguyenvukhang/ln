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

// surrounding git's colorful refs.
#define N_PRE_REFS sizeof("\x1b[m \x1b[33m")
#define N_POST_REFS sizeof("\x1b[m\x1b[33m)\x1b[m")

#define RESET "\x1b[m"

#define DARK_GRAY "\x1b[38;5;240m"
#define LIGHT_GRAY "\x1b[38;5;246m"

static int t; // time since unix epoch (in secs)

void print_refs(char *x, int len) {
  Print(DARK_GRAY "{");
  for (int i = 0; i < len; i++, x++) {
    if (strncmp(x, "origin", 6) == 0) {
      i += 6;
      x += 6;
      Print("*");
      continue;
    } else if (strncmp(x, "\x1b[33m", 5) == 0) {
      x[3] = '7'; // yellow -> gray.
    }
    print(x, 1);
  }
  Print(DARK_GRAY "}");
}

int print_time(const char *time_str) {
  int n = t - strtol(time_str, NULL, 10);
  static char buf[8];
#define send(l)                                                                \
  snprintf(buf, 8, "%d", n);                                                   \
  print(buf, strlen(buf));                                                     \
  return Print(l);
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
  const char *graph = buf, *hash;

  if (!(hash = strstr(buf, SEP)))
    return print(buf, strlen(buf));
  hash += SEP_L;

  char *refs = strstr(hash, SEP) + SEP_L;
  char *comment = strstr(refs, SEP) + SEP_L;
  char *time_str = strstr(comment, SEP) + SEP_L;

  int time = strtol(time_str, NULL, 10), refs_l;

  print(graph, hash - graph - SEP_L);
  Print("\x1b[33m");
  print(hash, refs - hash - SEP_L);
  Print(" ");

  if ((refs_l = comment - refs - SEP_L) > 8) {
    print_refs(refs + N_PRE_REFS, refs_l - N_PRE_REFS - N_POST_REFS + 1);
    Print(" ");
  }
  Print(RESET);

  print(comment, time_str - comment - SEP_L);
  Print(DARK_GRAY " (" LIGHT_GRAY);
  print_time(time_str);
  return Print(DARK_GRAY ")" RESET "\n");
}

int main(const int argc, const char **argv) {
  t = time(NULL);

  int p[2]; // [read, write]
  if (pipe(p) < 0) {
    perror("Unable to start pipe.");
    return -1;
  }

  // Start a fork for `git` where it writes to `p[1]`.
  if (fork() == 0) {
    close(p[0]);
    const char *args[argc + 8];
    int i = 0;
#define ARG(v) args[i++] = v
    ARG("git");
    ARG("log");
    for (int j = 1; j < argc; j++, i++)
      args[i] = argv[j];
    ARG("--color=always");
    ARG("--graph");
    ARG("--format=" SEP "%h" SEP "%C(auto)%d" SEP "%s" SEP "%at");
    ARG(NULL);
#undef ARG
    dup2(p[1], STDOUT_FILENO);
    execvp("git", (char *const *)args);
  }

  close(p[1]);
  dup2(p[0], STDIN_FILENO);

  // Process `git`'s output and write it to regular old stdout.
  if (system("which less > /dev/null 2>&1")) {
    char buf[BUF_SIZE];
    for (; fgets(buf, BUF_SIZE, stdin);)
      send_line(buf);
    return 0;
  }

  int q[2]; // [read, write]
  if (pipe(q) < 0) {
    perror("Unable to start pipe.");
    return -1;
  }

  // Process `git`'s output AND send it to `q` to pass to `less`.
  if (fork() == 0) {
    char buf[BUF_SIZE];
    close(q[0]);
    dup2(q[1], STDOUT_FILENO);
    for (; fgets(buf, BUF_SIZE, stdin);)
      send_line(buf);
    return 0;
  }
  dup2(q[0], STDIN_FILENO);
  close(q[1]);
  return execlp("less", "less", "-RF", NULL);
}
