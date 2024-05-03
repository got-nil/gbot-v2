
class DebugLogger:

    DEBUG: bool = False

    def debug(self, *args) -> None:
        if not self.DEBUG:
            return

        print(f"[{type(self).__name__}]", " -> ", *args)
