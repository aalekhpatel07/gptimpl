# GPTImpl

Ask ChatGPT to implement your Python functions for you.

## Usage

Suppose you have a Python file called `example.py` that contains an unimplemented functions as follows:
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

```shell
gptimpl example.py --overwrite
```

