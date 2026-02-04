import argparse
import json
import random
import statistics
import threading
import time
from concurrent.futures import ThreadPoolExecutor

import urllib.request
import urllib.error


def parse_args():
    parser = argparse.ArgumentParser(description="SurgeDB HTTP performance benchmark")
    parser.add_argument("--base-url", default="http://localhost:3000")
    parser.add_argument("--collection", default="perf_collection")
    parser.add_argument("--dimensions", type=int, default=384)
    parser.add_argument("--distance", default="Cosine")
    parser.add_argument("--quantization", default="None", choices=["None", "SQ8", "Binary"])
    parser.add_argument("--prefill", type=int, default=10000)
    parser.add_argument("--batch-size", type=int, default=200)
    parser.add_argument("--mode", default="mixed", choices=["search", "insert", "mixed"])
    parser.add_argument("--duration", type=int, default=60)
    parser.add_argument("--concurrency", type=int, default=32)
    parser.add_argument("--search-k", type=int, default=10)
    parser.add_argument("--search-ratio", type=float, default=0.7)
    parser.add_argument("--insert-ratio", type=float, default=0.2)
    parser.add_argument("--use-filter", action="store_true")
    parser.add_argument(
        "--filter-type",
        default="Exact",
        choices=["Exact", "OneOf", "Range"],
        help="Filter type when --use-filter is set",
    )
    parser.add_argument("--no-metadata", action="store_true")
    parser.add_argument("--output", default=None)
    parser.add_argument("--seed", type=int, default=42)
    return parser.parse_args()


def _post_json(url, payload, timeout=30):
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.getcode(), resp.read()


def _delete(url, timeout=30):
    req = urllib.request.Request(url, method="DELETE")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.getcode(), resp.read()


def create_collection(args):
    try:
        _delete(f"{args.base_url}/collections/{args.collection}")
    except urllib.error.HTTPError:
        pass
    payload = {
        "name": args.collection,
        "dimensions": args.dimensions,
        "distance_metric": args.distance,
    }
    if args.quantization != "None":
        payload["quantization"] = args.quantization
    status, body = _post_json(f"{args.base_url}/collections", payload)
    if status >= 400:
        raise RuntimeError(f"create collection failed: {status} {body.decode('utf-8', 'ignore')}")


def random_vector(dim, rng):
    return [rng.random() for _ in range(dim)]


def prefill(args):
    if args.prefill <= 0:
        return
    rng = random.Random(args.seed)
    total = args.prefill
    batch_size = max(1, args.batch_size)
    sent = 0
    while sent < total:
        batch = []
        for i in range(batch_size):
            if sent >= total:
                break
            idx = sent
            batch.append(
                {
                    "id": f"prefill_{idx}",
                    "vector": random_vector(args.dimensions, rng),
                    "metadata": {
                        "tag": "even" if idx % 2 == 0 else "odd",
                        "score": float(idx),
                    },
                }
            )
            sent += 1
        status, body = _post_json(
            f"{args.base_url}/collections/{args.collection}/vectors/batch",
            {"vectors": batch},
        )
        if status >= 400:
            raise RuntimeError(
                f"prefill failed: {status} {body.decode('utf-8', 'ignore')}"
            )


def build_filter_payload(filter_type):
    if filter_type == "Exact":
        return {"Exact": ["tag", "even"]}
    if filter_type == "OneOf":
        return {"OneOf": ["tag", ["even", "odd"]]}
    if filter_type == "Range":
        return {"Range": {"field": "score", "gt": 500.0}}
    return {"Exact": ["tag", "even"]}


def stats_from_latencies(latencies):
    if not latencies:
        return {}
    latencies_ms = [x * 1000.0 for x in latencies]
    latencies_ms.sort()
    def percentile(pct):
        if not latencies_ms:
            return None
        k = int((pct / 100.0) * (len(latencies_ms) - 1))
        return latencies_ms[k]

    return {
        "count": len(latencies_ms),
        "avg_ms": statistics.mean(latencies_ms),
        "p50_ms": percentile(50),
        "p95_ms": percentile(95),
        "p99_ms": percentile(99),
        "min_ms": latencies_ms[0],
        "max_ms": latencies_ms[-1],
    }


def print_summary(summary):
    search = summary.get("search") or {}
    insert = summary.get("insert") or {}

    def fmt(val):
        if val is None:
            return "-"
        if isinstance(val, float):
            return f"{val:.2f}"
        return str(val)

    print("\nSummary")
    print("metric,search,insert")
    for key in ["count", "qps", "avg_ms", "p50_ms", "p95_ms", "p99_ms", "min_ms", "max_ms"]:
        print(f\"{key},{fmt(search.get(key))},{fmt(insert.get(key))}\")


def main():
    args = parse_args()
    create_collection(args)
    prefill(args)

    stop_event = threading.Event()
    stats_lock = threading.Lock()

    latencies = {"search": [], "insert": []}
    errors = {"search": 0, "insert": 0}

    def worker(worker_id):
        rng = random.Random(args.seed + worker_id)
        while not stop_event.is_set():
            try:
                if args.mode == "search":
                    op = "search"
                elif args.mode == "insert":
                    op = "insert"
                else:
                    roll = rng.random()
                    if roll < args.search_ratio:
                        op = "search"
                    elif roll < args.search_ratio + args.insert_ratio:
                        op = "insert"
                    else:
                        op = "search"

                if op == "search":
                    payload = {
                        "vector": random_vector(args.dimensions, rng),
                        "k": args.search_k,
                    }
                    if args.no_metadata:
                        payload["include_metadata"] = False
                    if args.use_filter:
                        payload["filter"] = build_filter_payload(args.filter_type)
                    start = time.perf_counter()
                    status, _ = _post_json(
                        f"{args.base_url}/collections/{args.collection}/search",
                        payload,
                    )
                    elapsed = time.perf_counter() - start
                    if status >= 400:
                        with stats_lock:
                            errors["search"] += 1
                    else:
                        with stats_lock:
                            latencies["search"].append(elapsed)
                else:
                    payload = {
                        "id": f"insert_{worker_id}_{rng.randint(0, 1_000_000)}",
                        "vector": random_vector(args.dimensions, rng),
                        "metadata": {
                            "tag": "even" if rng.randint(0, 1) == 0 else "odd",
                            "score": rng.random() * 1000.0,
                        },
                    }
                    start = time.perf_counter()
                    status, _ = _post_json(
                        f"{args.base_url}/collections/{args.collection}/vectors",
                        payload,
                    )
                    elapsed = time.perf_counter() - start
                    if status >= 400:
                        with stats_lock:
                            errors["insert"] += 1
                    else:
                        with stats_lock:
                            latencies["insert"].append(elapsed)
            except (urllib.error.HTTPError, urllib.error.URLError):
                with stats_lock:
                    errors["search"] += 1

    start_time = time.perf_counter()
    with ThreadPoolExecutor(max_workers=args.concurrency) as executor:
        for i in range(args.concurrency):
            executor.submit(worker, i)

        while time.perf_counter() - start_time < args.duration:
            time.sleep(0.5)

        stop_event.set()

    duration = time.perf_counter() - start_time

    summary = {
        "duration_s": duration,
        "mode": args.mode,
        "dimensions": args.dimensions,
        "quantization": args.quantization,
        "prefill": args.prefill,
        "concurrency": args.concurrency,
        "include_metadata": not args.no_metadata,
        "filter_type": args.filter_type if args.use_filter else None,
        "search": stats_from_latencies(latencies["search"]),
        "insert": stats_from_latencies(latencies["insert"]),
        "errors": errors,
    }

    if summary["search"]:
        summary["search"]["qps"] = summary["search"]["count"] / duration
    if summary["insert"]:
        summary["insert"]["qps"] = summary["insert"]["count"] / duration

    output = json.dumps(summary, indent=2)
    print(output)

    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(output + "\n")

    print_summary(summary)


if __name__ == "__main__":
    main()
