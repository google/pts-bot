import functools
import unittest
import textwrap

COMMENT_WIDTH = 80 - 8  # 80 cols - 8 indentation space


def assert_description(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        description = textwrap.fill(
            kwargs["description"], COMMENT_WIDTH, replace_whitespace=False)
        docstring = textwrap.dedent(f.__doc__ or "")

        if docstring.strip() != description.strip():
            print(f'Expected description of {f.__name__}:')
            print(description)

            # Generate AssertionError
            test = unittest.TestCase()
            test.maxDiff = None
            test.assertMultiLineEqual(
                docstring.strip(),
                description.strip(),
                f'description does not match with function docstring of {f.__name__}')

        return f(*args, **kwargs)
    return wrapper


def format_function(id, description):
    wrapped = textwrap.fill(
        description, COMMENT_WIDTH, replace_whitespace=False)
    return (
        f'@assert_description\n'
        f'def {id}(self, **kwargs):\n'
        f'    """\n'
        f'{textwrap.indent(wrapped, "    ")}\n'
        f'    """\n'
        f'\n'
        f'    return "OK"\n'
    )


def format_proxy(profile, id, description):
    return (
        f'from ._description import assert_description\n'
        f'from ._proxy import ProfileProxy\n'
        f'\n'
        f'from blueberry.{profile.lower()}_grpc import {profile}\n'
        f'\n'
        f'\n'
        f'class {profile}Proxy(ProfileProxy):\n'
        f'\n'
        f'    def __init__(self, channel):\n'
        f'        super().__init__()\n'
        f'        self.{profile.lower()} = {profile}(channel)\n'
        f'\n'
        f'{textwrap.indent(format_function(id, description), "    ")}'
    )
