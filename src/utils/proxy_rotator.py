import os
import random
from threading import Lock
from typing import Literal, Optional


class ProxyRotator:
    """
    Manages a pool of proxy IPs and rotates between them to prevent rate limiting.
    Supports multiple rotation strategies and tracks proxy health.
    """

    def __init__(
        self,
        proxy_urls: list[str],
        strategy: Literal["round_robin", "random", "weighted"] = "round_robin",
        max_failures: int = 3,
    ):
        """
        Initialize the proxy rotator.

        :param proxy_urls: List of proxy URLs to rotate through
        :param strategy: Rotation strategy ("round_robin", "random", or "weighted")
        :param max_failures: Maximum failures before marking proxy as dead
        """
        if not proxy_urls:
            raise ValueError("proxy_urls cannot be empty")

        self.proxy_urls = proxy_urls
        self.strategy = strategy
        self.max_failures = max_failures
        self.current_index = 0
        self.lock = Lock()
        self.failed_proxies: set[str] = set()
        self.proxy_stats: dict[str, dict[str, int]] = {proxy: {"success": 0, "failures": 0} for proxy in proxy_urls}

    def get_proxy(self) -> Optional[str]:
        """
        Get next proxy based on configured strategy.

        :return: Proxy URL string or None if no proxies available
        """
        # Get available proxies (exclude failed ones)
        available = [p for p in self.proxy_urls if p not in self.failed_proxies]

        # If all proxies are failed, reset and try again
        if not available:
            self.failed_proxies.clear()
            available = self.proxy_urls

        # If still no proxies, return None
        if not available:
            return None

        # Select proxy based on strategy
        if self.strategy == "round_robin":
            with self.lock:
                proxy = available[self.current_index % len(available)]
                self.current_index += 1
            return proxy

        elif self.strategy == "random":
            return random.choice(available)

        elif self.strategy == "weighted":
            return self._get_weighted_proxy(available)

        else:
            # Default to round_robin if invalid strategy
            with self.lock:
                proxy = available[self.current_index % len(available)]
                self.current_index += 1
            return proxy

    def _get_weighted_proxy(self, proxies: list[str]) -> str:
        """
        Select proxy based on success rate (weighted selection).

        :param proxies: List of available proxy URLs
        :return: Selected proxy URL
        """
        if len(proxies) == 1:
            return proxies[0]

        weights = []
        for proxy in proxies:
            stats = self.proxy_stats[proxy]
            total = stats["success"] + stats["failures"]
            # If no history, give equal weight (0.5)
            success_rate = stats["success"] / total if total > 0 else 0.5
            weights.append(success_rate)

        # Normalize weights to avoid zero weights
        min_weight = min(weights)
        if min_weight <= 0:
            weights = [w - min_weight + 0.1 for w in weights]

        return random.choices(proxies, weights=weights)[0]

    def mark_success(self, proxy: str) -> None:
        """
        Mark a proxy as successful.

        :param proxy: Proxy URL that succeeded
        """
        if proxy in self.proxy_stats:
            self.proxy_stats[proxy]["success"] += 1
            # Remove from failed list if present (proxy recovered)
            if proxy in self.failed_proxies:
                self.failed_proxies.remove(proxy)

    def mark_failure(self, proxy: str) -> None:
        """
        Mark a proxy as failed. If failures exceed threshold, mark as dead.

        :param proxy: Proxy URL that failed
        """
        if proxy in self.proxy_stats:
            self.proxy_stats[proxy]["failures"] += 1
            # If failures exceed threshold, add to failed set
            if self.proxy_stats[proxy]["failures"] >= self.max_failures:
                self.failed_proxies.add(proxy)

    def get_stats(self) -> dict[str, dict[str, int]]:
        """
        Get statistics for all proxies.

        :return: Dictionary mapping proxy URLs to their stats
        """
        return self.proxy_stats.copy()

    def reset_stats(self) -> None:
        """
        Reset all statistics and failed proxies (useful for recovery scenarios).
        """
        self.failed_proxies.clear()
        self.proxy_stats = {proxy: {"success": 0, "failures": 0} for proxy in self.proxy_urls}
        self.current_index = 0


def create_proxy_rotator_from_env() -> Optional["ProxyRotator"]:
    """
    Create a ProxyRotator instance from environment variables.

    Checks PROXY_POOL first (comma-separated), then falls back to PROXY_URL.

    :return: ProxyRotator instance or None if proxies not configured
    """
    if os.getenv("USE_PROXY", "False").lower() != "true":
        return None

    # Check for PROXY_POOL first (multiple proxies)
    proxy_pool = os.getenv("PROXY_POOL", "").strip()
    if proxy_pool:
        proxy_urls = [url.strip() for url in proxy_pool.split(",") if url.strip()]
        if proxy_urls:
            strategy = os.getenv("PROXY_ROTATION_STRATEGY", "round_robin")
            max_failures = int(os.getenv("PROXY_MAX_FAILURES", "3"))
            return ProxyRotator(proxy_urls=proxy_urls, strategy=strategy, max_failures=max_failures)

    # Fallback to single PROXY_URL
    proxy_url = os.getenv("PROXY_URL", "").strip()
    if proxy_url:
        strategy = os.getenv("PROXY_ROTATION_STRATEGY", "round_robin")
        max_failures = int(os.getenv("PROXY_MAX_FAILURES", "3"))
        return ProxyRotator(proxy_urls=[proxy_url], strategy=strategy, max_failures=max_failures)

    return None
