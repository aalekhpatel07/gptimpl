import pathlib
import ast
import typing
from dataclasses import dataclass
from ast import FunctionDef


@dataclass(init=True)
class Function:
    source: str
    name: str

    @property
    def quoted(self) -> str:
        return "\n".join(("```python", self.source, "```"))


class FunctionCollector(ast.NodeVisitor):
    filename: pathlib.Path
    tree: ast.AST
    source: str
    source_lines: typing.List[str]

    functions: typing.List[Function]

    def __init__(self, filename: pathlib.Path, source=None) -> None:
        self.filename = filename
        self.source = source
        self.functions = []
        if source is None:
            self.load_file()

        self.source_lines = self.source.split("\n")
        try:
            self.tree = ast.parse(self.source)
        except SyntaxError:
            self.tree = ast.parse(self.source.replace("```", ""))

    def load_file(self):
        with open(self.filename, "r") as f:
            contents = f.read()
            self.source = contents

    def visit_FunctionDef(self, node: FunctionDef) -> typing.Any:
        function_start_row = node.lineno - 1
        function_end_row = node.end_lineno - 1
        function_start_col = node.col_offset
        function_end_col = node.end_col_offset

        first_line = self.source_lines[function_start_row][function_start_col:]
        full_lines = self.source_lines[function_start_row + 1:function_end_row]
        last_line = self.source_lines[function_end_row][:function_end_col]
        all_lines = [first_line, *full_lines, last_line]

        self.functions.append(
            Function(
                source="\n".join(all_lines),
                name=node.name
            )
        )

    def check(self):
        self.visit(self.tree)


def sanitize_codegen(contents: str) -> typing.List[Function]:
    """
    Extract the one and only function definition from the returned source.
    """
    collector = FunctionCollector(
        filename=pathlib.Path("<test>"),
        source=contents
    )
    collector.check()

    return collector.functions
