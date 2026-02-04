#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
import sys
import time
import urllib.request
import tomllib
from datetime import datetime, timezone


def parse_args():
    parser = argparse.ArgumentParser(description="Run SurgeDB perf harness")
    parser.add_argument("--config", default="scripts/perf/perf.toml")
    parser.add_argument("--output", default="scripts/perf/perf_report.json")
    parser.add_argument("--skip-core", action="store_true")
    return parser.parse_args()


def wait_for_health(base_url, health_path, timeout_secs):
    deadline = time.time() + timeout_secs
    url = base_url.rstrip("/") + health_path
    while time.time() < deadline:
        try:
            req = urllib.request.Request(url, method="GET")
            with urllib.request.urlopen(req, timeout=2) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(1)
    return False


def run_cmd(cmd, env=None):
    return subprocess.run(cmd, shell=True, env=env, capture_output=True, text=True)


def run_http_scenario(base_url, http_cfg, scenario, output_dir):
    args = [
        sys.executable,
        "scripts/perf/http_bench.py",
        "--base-url",
        base_url,
        "--collection",
        http_cfg["collection"],
        "--dimensions",
        str(http_cfg["dimensions"]),
        "--prefill",
        str(http_cfg["prefill"]),
        "--duration",
        str(http_cfg["duration"]),
        "--concurrency",
        str(http_cfg["concurrency"]),
        "--mode",
        scenario.get("mode", "search"),
    ]

    if scenario.get("use_filter"):
        args.append("--use-filter")
        if scenario.get("filter_type"):
            args.extend(["--filter-type", scenario["filter_type"]])

    if scenario.get("no_metadata"):
        args.append("--no-metadata")

    output_path = os.path.join(output_dir, f"{scenario['name']}.json")
    args.extend(["--output", output_path])

    proc = subprocess.run(args, capture_output=True, text=True)
    result = {
        "name": scenario["name"],
        "cmd": " ".join(args),
        "returncode": proc.returncode,
        "stdout_tail": "\n".join(proc.stdout.splitlines()[-10:]),
        "stderr_tail": "\n".join(proc.stderr.splitlines()[-10:]),
    }
    if proc.returncode == 0 and os.path.exists(output_path):
        with open(output_path, "r", encoding="utf-8") as f:
            result["metrics"] = json.load(f)
    return result


def main():
    args = parse_args()
    with open(args.config, "rb") as f:
        cfg = tomllib.load(f)

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "config": cfg,
        "http": [],
        "core": [],
    }

    server_cfg = cfg["server"]
    base_url = server_cfg.get("base_url", "http://localhost:3000")
    health_path = server_cfg.get("health_path", "/health")
    timeout_secs = int(server_cfg.get("startup_timeout_secs", 60))

    server_proc = subprocess.Popen(server_cfg["start_cmd"], shell=True)
    try:
        ready = wait_for_health(base_url, health_path, timeout_secs)
        if not ready:
            print("Server failed to become healthy in time.")
            server_proc.terminate()
            report["error"] = "server_not_ready"
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(report, f, indent=2)
            return 1

        http_cfg = cfg["http"]
        output_dir = os.path.join(os.path.dirname(args.output), "_scenario_outputs")
        os.makedirs(output_dir, exist_ok=True)

        for scenario in http_cfg.get("scenarios", []):
            report["http"].append(run_http_scenario(base_url, http_cfg, scenario, output_dir))

        core_cfg = cfg.get("core", {})
        if core_cfg.get("enabled", True) and not args.skip_core:
            for cmd in core_cfg.get("bench_cmds", []):
                res = run_cmd(cmd)
                report["core"].append(
                    {
                        "cmd": cmd,
                        "returncode": res.returncode,
                        "stdout_tail": "\n".join(res.stdout.splitlines()[-20:]),
                        "stderr_tail": "\n".join(res.stderr.splitlines()[-20:]),
                    }
                )
    finally:
        server_proc.terminate()
        try:
            server_proc.wait(timeout=5)
        except Exception:
            server_proc.kill()

    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    print(f"Wrote report to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
