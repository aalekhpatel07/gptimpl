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
