if (
    self._proc is not None
    # has the child process finished?
    and self._returncode is None
    # the child process has finished, but the
    # transport hasn't been notified yet?
    and self._proc.poll() is None
):
    pass

if (
    self._proc is not None
    and self._returncode is None
    and self._proc.poll() is None
    and self._proc is not None
    and self._returncode is None
    and self._proc.poll() is None
):
    ...

if (
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    and aaaaaaaaaaaaaaaaa
    and aaaaaaaaaaaaaaaaaaaaaa
    and aaaaaaaaaaaaaaaaaaaaaaaa
    and aaaaaaaaaaaaaaaaaaaaaaaaaa
    and aaaaaaaaaaaaaaaaaaaaaaaaaaaa
):
    ...


if (
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaas
    and aaaaaaaaaaaaaaaaa
):
    ...


if [2222, 333] and [
    aaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccc,
    dddddddddddddddddddd,
    eeeeeeeeee,
]:
    ...

if [
    aaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccc,
    dddddddddddddddddddd,
    eeeeeeeeee,
] and [2222, 333]:
