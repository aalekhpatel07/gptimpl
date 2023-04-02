#!/usr/bin/env python3
import ast
import sys
import typing
import argparse
import pathlib
from dataclasses import dataclass
import astor
import logging

from .collector import FunctionCollector, Function
from .ai_codegen import (
    request_code_snippets
)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class Opts:
    files: typing.List[pathlib.Path]
    verbose: int
    overwrite: bool = False
    model: typing.Optional[str] = None


def parse_args() -> Opts:
    parser = argparse.ArgumentParser(
        description="Extract a collection of functions from Python files. Provide an OpenAI api key with "
                    "OPENAI_API_KEY env var. "
    )

    parser.add_argument(
        "files",
        metavar="FILE",
        nargs='+',
        help="The Python files to process.",
        type=pathlib.Path,
    )

    parser.add_argument(
        "-v",
        "--verbose",
        action='count',
        default=0,
        help="Specify the verbosity of the output."
    )

    parser.add_argument(
        "-w",
        "--overwrite",
        action="store_true",
        help="If specified, the generated files will be overwritten instead of logging to stdout."
    )

    parser.add_argument(
        "-m",
        "--model",
        type=str,
        help="Select the backend model to use. Defaults to gpt-3.5-turbo",
        default=None
    )

    opts = parser.parse_args()
    return Opts(
        verbose=opts.verbose,
        files=opts.files,
        overwrite=opts.overwrite,
        model=opts.model
    )


class FunctionImplementor(ast.NodeTransformer):
    file: pathlib.Path
    implementations: typing.Dict[str, str] = dict()

    def __init__(self, file: pathlib.Path, implementations: typing.Dict[str, str]):
        self.file = file
        self.implementations = implementations

    def visit_FunctionDef(self, node: ast.FunctionDef) -> typing.Any:
        parsed = ast.parse(self.implementations[node.name])
        node.body = parsed.body[0].body
        return node

    def generate_replacement(self) -> str:
        with open(self.file, "r") as f:
            tree = ast.parse(f.read())
        self.visit(tree)

        return astor.to_source(tree)


def extract_functions(files: typing.List[pathlib.Path]) -> typing.Dict[pathlib.Path, typing.Dict[str, Function]]:
    results = dict()
    for file in files:
        analyzer = FunctionCollector(filename=file)
        analyzer.check()
        for function in analyzer.functions:
            if file not in results:
                results[file] = dict()
            results[file][function.name] = function
    return results


def request_codegen(
    extracted_functions: typing.Dict[pathlib.Path, typing.Dict[str, Function]],
    **kwargs
) -> typing.Dict[pathlib.Path, typing.Dict[str, str]]:

    function_sources = []

    for (_filename, functions) in extracted_functions.items():
        for (function_name, function_body) in functions.items():
            function_sources.append(function_body.quoted)
    implementations = request_code_snippets(function_sources, **kwargs)

    impl_iter = iter(implementations)
    implementation_map = dict()

    for (filename, functions) in extracted_functions.items():
        if filename not in implementation_map:
            implementation_map[filename] = dict()
        for (function_name, _) in functions.items():
            implementation_map[filename][function_name] = next(impl_iter)

    return implementation_map


def main():
    opts = parse_args()
    if opts.verbose > 0:
        logger.setLevel(logging.DEBUG)

    logger.debug("Extracting functions...")
    extracted_functions = extract_functions(opts.files)
    logger.debug("Waiting for codegen...")
    if opts.model is not None:
        logger.debug("requested model: %s", opts.model)
    impls = request_codegen(extracted_functions, model=opts.model)
    logger.debug("Updating the nodes...")

    replacements = dict()

    for file in impls.keys():
        function_implementor = FunctionImplementor(
            file=file,
            implementations=impls[file]
        )
        replacement = function_implementor.generate_replacement()
        replacements[file] = replacement

    for (filename, replacement_value) in replacements.items():
        logger.debug("Generated replacement for: %s", filename)

        if opts.overwrite:
            logger.debug("Writing replacement to: %s", filename)
            with open(filename, "w") as f:
                f.write(replacement_value)
        else:
            sys.stdout.write(replacement_value)


if __name__ == '__main__':
    main()
