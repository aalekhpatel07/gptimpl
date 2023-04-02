# GPTImpl

##### Ask ChatGPT to implement your Python functions for you.

## Installation
___

A distribution is available on PyPI.
```shell
pip install gptimpl
```

## Usage
___

Since we interact with OpenAI, the OpenAI API key
is a run-time dependency for `gptimpl`. You can either provide it per-invocation, i.e.
```shell
OPENAI_API_KEY="<key>" gptimpl -v example.py
```
or you can export it before running any `gptimpl` commands with:
```shell
export OPENAI_API_KEY="<key>"
```

```shell
gptimpl --help
```
```
usage: gptimpl [-h] [-v] [-w] FILE [FILE ...]

Extract a collection of functions from Python files. Provide an OpenAI api key with OPENAI_API_KEY env var.

positional arguments:
  FILE             The Python files to process.

optional arguments:
  -h, --help       show this help message and exit
  -v, --verbose    Specify the verbosity of the output.
  -w, --overwrite  If specified, the generated files will be overwritten instead of logging to stdout.
```

**DISCLAIMER**: Please use the `-w` flag only if you want to overwrite your source files and they are already version controlled!
The GPT generated code can be garbage sometimes so it is recommended to overwrite only if you have a way to revert the changes.

### Updating the function body in-place
Suppose you have a Python file called `example.py` that contains unimplemented functions as follows:
    
```python
def fibonacci(n: int) -> int:
    """
    Return the n-th fibonacci number.
    """

def estimate_pi(n: int) -> float:
    """
    Estimate Pi using Gregory-Leibniz series
    using the first n terms.
    """
```
Then we can pass this file through `gptimpl` to generate the implementations for us.

**Note**: Without `--overwrite`, it defaults to printing the replacement contents to stdout.
```shell
gptimpl example.py -v --overwrite
```

This overwrites the file with the following patch:

```diff
-def fibonacci(n: int) -> int:
+def fibonacci(n: int) ->int:
     """
     Return the nth fibonacci number.
     """
+    if n <= 1:
+        return n
+    else:
+        return fibonacci(n - 1) + fibonacci(n - 2)
 
 
-def estimate_pi(n: int) -> float:
+def estimate_pi(n: int) ->float:
     """
     Estimate Pi using Gregory-Leibniz series
     using the first n terms.
     """
+    pi = 0
+    for i in range(n):
+        pi += (-1) ** i / (2 * i + 1)
+    return pi * 4
```

resulting in an `example.py` that looks like:
```python
def fibonacci(n: int) ->int:
    """
    Return the nth fibonacci number.
    """
    if n <= 1:
        return n
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)


def estimate_pi(n: int) ->float:
    """
    Estimate Pi using Gregory-Leibniz series
    using the first n terms.
    """
    pi = 0
    for i in range(n):
        pi += (-1) ** i / (2 * i + 1)
    return pi * 4
```