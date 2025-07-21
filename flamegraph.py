import matplotlib.pyplot as plt
import matplotlib.cm as cm
import pandas as pd
import numpy as np
import json

f = open('shared/execution_times.json', 'r')
data = json.load(f)

# Convert to DataFrame for easier handling
time_df = pd.json_normalize(data, meta=["function_name", "start_time", "end_time", "duration"])

# Function to assign depth based on time range overlaps
def assign_depth(df):
    df = df.sort_values('start_time').reset_index(drop=True)
    depth = [0.0] * len(df)  # Initialize depth column

    # Step 1: Assign depths based on containment
    for i in range(len(df)):
        current_start = df.iloc[i]['start_time']
        current_end = df.iloc[i]['end_time']
        max_depth = 0.0

        # Check for functions that contain the current function
        for j in range(len(df)):
            if i != j:
                other_start = df.iloc[j]['start_time']
                other_end = df.iloc[j]['end_time']
                # If current function is fully contained within another function
                if other_start <= current_start and current_end <= other_end:
                    max_depth = max(max_depth, depth[j] + 0.1)

        depth[i] = max_depth

    df['depth'] = depth

    # Step 2: Identify depth=0 functions that contain no other functions

    rows_to_keep = []
    for i in range(len(df)):
        if df.iloc[i]['depth'] > 0:
            # Keep all non-depth=0 functions
            rows_to_keep.append(i)
        else:
            # For depth=0 functions, check if they contain any other function
            current_start = df.iloc[i]['start_time']
            current_end = df.iloc[i]['end_time']
            contains_others = False
            for j in range(len(df)):
                if i != j:
                    other_start = df.iloc[j]['start_time']
                    other_end = df.iloc[j]['end_time']
                    # Check if this depth=0 function contains another function
                    if current_start <= other_start and other_end <= current_end:
                        contains_others = True
                        break
            if contains_others:
                rows_to_keep.append(i)

    # Step 3: Filter DataFrame to keep only valid rows
    df = df.iloc[rows_to_keep].reset_index(drop=True)


    return df

# Assign depth
time_df = assign_depth(time_df)

f = open('shared/allocations.json', 'r')
data = json.load(f)

def assign_allocations(df):
    memory_usage = 0
    cumulative_memory = []
    timestamps = []

    for _, row in df.iterrows():
        if row['alloc_type'] == 'Malloc':
            memory_usage += row['size']
        elif row['alloc_type'] == 'Free':
            memory_usage -= row['size']
        cumulative_memory.append(memory_usage)
        timestamps.append(row['timestamp'])

    return cumulative_memory,timestamps

# Convert to DataFrame for easier handling
allocations_df = pd.json_normalize(data, meta=["size", "alloc_type", "timestamp"])
cumulative_memory,timestamps = assign_allocations(allocations_df)

# Create figure and axis
fig, ax = plt.subplots(figsize=(10, 5))

ax2 = ax.twinx()
ax2.step(timestamps, cumulative_memory, marker='o', linestyle='--', color='b', alpha=0.5, label="Memory")

# Assign colors to functions (optional: use a colormap or hash function names)
norm = plt.Normalize(time_df['depth'].min(), time_df['depth'].max())
cmap = cm.autumn.reversed()
#cmap = cm.winter

# Plot each function call as a horizontal bar
for i, row in time_df.iterrows():
    color = cmap(norm(row['depth']))
    ax.barh(
        row["depth"],
        row["duration"],
        left=row["start_time"],
        height=0.1,
        color=color,
        edgecolor="black",
        label=row["function_name"]
    )

# Customize the plot
ax.set_xlabel("Time (ns)")
ax.set_ylabel("Call Stack Depth")
ax.set_yticklabels([])
ax.set_title("Flamegraph")
ax2.yaxis.tick_right()
ax2.yaxis.set_label_position("right")
ax2.set_ylabel("Memory usage (Bytes)")

# Optional: Add function names as text labels on bars
for i, row in time_df.iterrows():
    ax.text(
        row["start_time"] + row["duration"] / 2,
        row["depth"],
        row["function_name"],
        ha="center",
        va="center",
        color="black"
    )

# Remove duplicate labels in legend
handles, labels = ax.get_legend_handles_labels()
handle, label = ax2.get_legend_handles_labels()
by_label = dict(zip(labels + label, handles + handle))
ax.legend(by_label.values(), by_label.keys(), loc="upper left")

plt.tight_layout()
plt.show()