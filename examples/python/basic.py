from surgedb import SurgeClient, SurgeConfig, DistanceMetric
import json


def main():
    # 1. Initialize in-memory database
    client = SurgeClient.new_in_memory(dimensions=3)

    print("SurgeDB Python Example")
    print("----------------------")

    # 2. Insert vectors
    # Note: metadata must be a JSON string
    client.insert(
        "apple", [1.0, 0.0, 0.0], json.dumps({"type": "fruit", "color": "red"})
    )
    client.insert(
        "banana", [0.0, 1.0, 0.0], json.dumps({"type": "fruit", "color": "yellow"})
    )
    client.insert(
        "truck", [0.0, 0.0, 1.0], json.dumps({"type": "vehicle", "color": "blue"})
    )

    print(f"Inserted {client.len()} vectors.")

    # 3. Search
    query = [0.9, 0.1, 0.0]
    print(f"\nSearching for: {query}")

    results = client.search(query, k=1)

    if results:
        res = results[0]
        print("Match Found!")
        print(f"ID: {res.id}")
        print(f"Score: {res.score:.4f}")
        if res.metadata_json:
            print(f"Metadata: {json.loads(res.metadata_json)}")


if __name__ == "__main__":
    main()
