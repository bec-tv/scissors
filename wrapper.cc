#include "wrapper.h"
#include "QApplication"
#include "QThread"

typedef std::function<void()> VoidFunc;

extern "C" {
  void scissors_set_ui_task_handler(obs_task_handler_t task);
}

static inline Qt::ConnectionType WaitConnection() {
  return QThread::currentThread() == qApp->thread()
    ? Qt::DirectConnection
    : Qt::BlockingQueuedConnection;
}


static void ui_task_handler(obs_task_t task, void *param, bool wait) {
  auto doTask = [=]() {
    /* to get clang-format to behave */
    task(param);
  };
  QMetaObject::invokeMethod(qApp, "Exec",
    wait ? WaitConnection() : Qt::AutoConnection,
    Q_ARG(VoidFunc, doTask));
}

extern "C" {
  void scissors_vec2_set(struct vec2 *dst, float x, float y) {
    vec2_set(dst, x, y);
  }

  void scissors_run_qt() {
    int argc = 1;
    char *argv[] = { "scissors.exe" };
    new QApplication(argc, argv);

    scissors_set_ui_task_handler(ui_task_handler);
  }
}