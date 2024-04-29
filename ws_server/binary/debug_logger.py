
class DebugLogger:

    DEBUG: bool = True

    def debug(self, *args) -> None:
        if not self.DEBUG:
            return

        print(f"[{type(self).__name__}]", " -> ", *args)
