#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define print(text, len) write(STDOUT_FILENO, text, len)
#define SEP " :_|"
#define SEP_L (sizeof(SEP) - 1);
#define BUF_SIZE 0x200

#define NORMAL() print("\x1b[0m", 4)
#define YELLOW() print("\x1b[33m", 5)
#define DARK_GRAY() print("\x1b[38;5;241m", 11)
#define LIGHT_GRAY() print("\x1b[38;5;246m", 11)

static int t; // time since unix epoch (in secs)

void print_refs(const char *x, int len) {
  LIGHT_GRAY();
  print("(", 1);
  for (int i = 0; i < len; i++) {
    if (strncmp(&x[i], "origin", 6) == 0) {
      i += 6;
      print("*", 1);
      continue;
    }
    print(&x[i], 1);
  }
  LIGHT_GRAY();
  print(")", 1);
  NORMAL();
}

int print_time(const char *time_str) {
  int n = t - strtol(time_str, NULL, 10);
  static char buf[8];
#define send(l)                                                                \
  snprintf(buf, 8, "%d", n);                                                   \
  print(buf, strlen(buf));                                                     \
  return print(l, 1);
  // clang-format off
  if (n < 60)         { send("s"); }
  if ((n /= 60) < 60) { send("m"); }
  if ((n /= 60) < 24) { send("h"); }
  if ((n /= 24) < 7)  { send("d"); }
  // clang-format on
  send("w");
#undef send
}

int send_line(const char *buf) {
  const char *graph = buf, *hash;

  if (!(hash = strstr(buf, SEP)))
    return print(buf, strlen(buf));
  hash += SEP_L;

  const char *refs = strstr(hash, SEP) + SEP_L;
  const char *comment = strstr(refs, SEP) + SEP_L;
  const char *time_str = strstr(comment, SEP) + SEP_L;

  int time = strtol(time_str, NULL, 10);

  int graph_l = hash - buf - SEP_L;
  int hash_l = refs - hash - SEP_L;
  int refs_l = comment - refs - SEP_L;
  int comment_l = time_str - comment - SEP_L;

  print(graph, graph_l);
  YELLOW();
  print(hash, hash_l + 1);
  NORMAL();

  if (refs_l > 6) {
    print_refs(refs + 10, refs_l - 25);
    print(" ", 1);
  }

  print(comment, comment_l);
  DARK_GRAY();
  print(" (", 2);
  LIGHT_GRAY();
  print_time(time_str);
  DARK_GRAY();
  print(")", 1);
  NORMAL();
  return print("\n", 1);
}

int main(int argc, const char **argv) {
  t = time(NULL);

  int p[2]; // [read, write]
  if (pipe(p) < 0) {
    perror("Unable to start pipe.");
    return -1;
  }

  // Start a fork for `git` where it writes to `p[1]`.
  if (fork() == 0) {
    close(p[0]);
    const char *args[argc + 1 + 4];
    args[0] = "git";
    args[1] = "log";
    for (int i = 1; i < argc; i++)
      args[i + 1] = argv[i];
    args[++argc] = "--color=always";
    args[++argc] = "--graph";
    args[++argc] =
        "--format=" SEP "%h" SEP "%C(auto)%d%Creset" SEP "%s" SEP "%at";
    args[++argc] = NULL;
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
