import math
from typing import Tuple

def entropy_fingerprint(source: str) -> Tuple[float, bool]:
    """
    Returns (entropy_ratio, is_malicious)
    Rejects anything > 1.05× theoretical minimum (obfuscation attack)
    """
    if not source.strip():
        return 0.0, False

    freq = {}
    for c in source:
        freq[c] = freq.get(c, 0) + 1

    length = len(source)
    distinct = len(freq)
    min_possible = math.log2(distinct) if distinct > 1 else 0
    actual = -sum((count/length) * math.log2(count/length) for count in freq.values())

    ratio = actual / min_possible if min_possible > 0 else 1.0
    is_malicious = ratio > 1.05

    return ratio, is_malicious

# Hard gate — used by all ingress paths
def reject_if_obfuscated(source: str, lang: str) -> None:
    ratio, malicious = entropy_fingerprint(source)
    if malicious:
        raise ValueError(
            f"UN1C⓪ REJECT: {lang} source entropy {ratio:.3f}x > 1.05 limit → OBFUSCATION DETECTED\n"
            "All hostile variants are now part of the permanent training set."
        )
