from mmi2grpc._description import format_function


class ProfileProxy:

    def interact(self, id: str, test: str, description: str, pts_addr: bytes):
        try:
            return getattr(self, id)(
                test=test, description=description, pts_addr=pts_addr)
        except AttributeError:
            code = format_function(id, description)
            assert False, f'Unhandled mmi {id}\n{code}'
