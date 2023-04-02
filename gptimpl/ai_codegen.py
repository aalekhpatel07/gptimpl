import typing
import re
import os

import openai

from .collector import sanitize_codegen


CODE_SNIPPET_RE: re.Pattern = re.compile(r"(?s)```python(.+)```")


class CodeGenFailedException(Exception):
    """Raised when the code gen engine fails to answer the question."""


def request(
    premise: str,
    question: str,
    model: typing.Optional[str] = None,
) -> str:
    if model is None:
        model = "gpt-3.5-turbo"
    openai.api_key = os.getenv("OPENAI_API_KEY")
    response = openai.ChatCompletion.create(
        model=model,
        messages=[
            {
                "role": "system", "content": premise
            },
            {
                "role": "user", "content": question
            }
        ]
    )

    choices = response.get('choices', [])
    if len(choices):
        first_choice = choices[0]
        return first_choice['message']['content']

    raise CodeGenFailedException(response)


def request_code_snippets(questions: typing.List[str], **kwargs) -> typing.List[str]:
    response = request(
        question="\n".join(questions),
        premise="Please complete the implementations of the following Python functions in a single code snippet:\n"
    )
    match = CODE_SNIPPET_RE.search(response)
    if match is not None:
        snippet = match.group(1).strip()
        return [function.source for function in sanitize_codegen(snippet)]
