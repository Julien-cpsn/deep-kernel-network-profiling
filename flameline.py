import matplotlib.pyplot as plt
import matplotlib.cm as cm
import pandas as pd
import numpy as np
import argparse
import pathlib
import math
import json
import sys

def assign_allocations(df):
    memory_usage = {'kmalloc': 0, 'kmem_cache': 0, 'total': 0}
    cumulative_memory = {'kmalloc': [0], 'kmem_cache': [0], 'total': [0]}
    timestamps = {'kmalloc': [0], 'kmem_cache': [0], 'total': [0]}

    # Sort by timestamp to ensure chronological order
    df = df.sort_values('timestamp').reset_index(drop=True)

    for _, row in df.iterrows():
        alloc_type = row['alloc_type']
        alloc_direction = row['alloc_direction']
        size = row['size']

        # Update memory usage for the specific memory_type and total
        if alloc_direction == 'Alloc':
            memory_usage[alloc_type] += size
            memory_usage['total'] += size
        elif alloc_direction == 'Free':
            memory_usage[alloc_type] -= size
            memory_usage['total'] -= size

        # Append to respective lists
        # For kmem and kmem_cache, append only if the row matches the memory_type
        if alloc_type == 'kmalloc':
            cumulative_memory['kmalloc'].append(memory_usage['kmalloc'])
            timestamps['kmalloc'].append(row['timestamp'])
        elif alloc_type == 'kmem_cache':
            cumulative_memory['kmem_cache'].append(memory_usage['kmem_cache'])
            timestamps['kmem_cache'].append(row['timestamp'])

        # Always append to total
        cumulative_memory['total'].append(memory_usage['total'])
        timestamps['total'].append(row['timestamp'])

    return cumulative_memory, timestamps


# Function to update text visibility based on zoom
def update_text_visibility(event_ax, fig, ax):
    x_min, x_max = event_ax.get_xlim()
    visible_range = x_max - x_min
    threshold = visible_range * 0.025

    for text, duration, start_time, end_time in texts:
        is_visible = (start_time <= x_max and end_time >= x_min) and (duration >= threshold)
        text.set_visible(is_visible)

    fig.canvas.restore_region(fig.canvas.copy_from_bbox(ax.bbox))
    for text, _, _, _ in texts:
        if text.get_visible():
            ax.draw_artist(text)
    fig.canvas.blit(ax.bbox)

def plot_stack_merged(time_df, ax, time_filter):
    # Assign colors to functions (optional: use a colormap or hash function names)
    norm = plt.Normalize(0, 100)
    cmap = cm.autumn.reversed()
    #cmap = cm.rainbow

    # Plot each function call as a horizontal bar
    for i, row in time_df.sort_values(by=['function_name']).iterrows():
        color = cmap(norm(row['inner_duration_perc']))
        ax.barh(
            row["depth"],
            row["duration"],
            left=row["start_time"],
            height=0.1,
            color=color,
            edgecolor=(0.0, 0.0, 0.0, 0.5),
            label=row["function_name"]
        )

    for i, row in time_df.iterrows():
        if row["duration"] < 10000:
            text = ax.text(
                row["start_time"] + row["duration"] / 2,
                row["depth"],
                row["function_name"],
                ha="center",
                va="center",
                color="black",
                rotation="vertical",
                size="small",
                visible=False
            )
        else:
            text = ax.text(
                row["start_time"] + row["duration"] / 2,
                row["depth"],
                row["function_name"],
                ha="center",
                va="center",
                color="black",
                size="small",
                visible=False
            )

        texts.append((text, row['duration'], row['start_time'], row['end_time']))

    ax.set_xlim(left=0)
    ax.set_ylabel("Call Stack Depth")
    ax.set_yticklabels([])

def plot_stack_per_cpu(time_df, cpuid, ax, time):
    # Assign colors to functions (optional: use a colormap or hash function names)
    norm = plt.Normalize(0, 100)
    cmap = cm.autumn.reversed()

    inner_time_df = time_df.query(f'cpuid == {cpuid}')

    # Plot each function call as a horizontal bar
    for i, row in inner_time_df.sort_values(by=['function_name']).iterrows():
        color = cmap(norm(row['inner_duration_perc']))
        ax.barh(
            row["depth"],
            row["duration"],
            left=row["start_time"],
            height=0.1,
            color=color,
            edgecolor=(0.0, 0.0, 0.0, 0.5),
            label=row["function_name"]
        )

    ax.set_title(f"CPU {cpuid}")
    ax.set_ylabel("Call Stack Depth")
    ax.set_yticklabels([])

    # Optional: Add function names as text labels on bars
    for i, row in inner_time_df.iterrows():
        if row["duration"] < 10000:
            text = ax.text(
                row["start_time"] + row["duration"] / 2,
                row["depth"],
                row["function_name"],
                ha="center",
                va="center",
                color="black",
                rotation="vertical",
                size="small",
                visible=False
            )
        else:
            text = ax.text(
                row["start_time"] + row["duration"] / 2,
                row["depth"],
                row["function_name"],
                ha="center",
                va="center",
                color="black",
                size="small",
                visible=False
            )

        texts.append((text, row['duration'], row['start_time'], row['end_time']))

def plot_memory(data, ax, time_filter):
    allocations_df = pd.json_normalize(data['allocations'], meta=["alloc_type", "alloc_direction", "size", "timestamp"])

    if time_filter is not None:
        allocations_df = allocations_df.query(f'timestamp >= {time_filter[0]} and timestamp <= {time_filter[1]}')
        allocations_df.loc[:, 'timestamp'] -= time_filter[0]

    cumulative_memory, timestamps = assign_allocations(allocations_df)

    ax.yaxis.tick_right()
    ax.set_ylabel("Memory usage\n(Bytes)")

    ax.step(timestamps['kmalloc'], cumulative_memory['kmalloc'], where='post', marker='o', linestyle='--', color='red', alpha=0.8, label='kmalloc')
    ax.step(timestamps['kmem_cache'], cumulative_memory['kmem_cache'], where='post', marker='o', linestyle='--', color='blue', alpha=0.8, label='kmem_cache')
    ax.step(timestamps['total'], cumulative_memory['total'], where='post', marker='o', linestyle='-', color='black', alpha=1, label='Total Memory')
    ax.grid(True, linestyle='--', alpha=0.7)

    handles, labels = ax.get_legend_handles_labels()
    by_label = dict(zip(labels, handles))
    ax.legend(by_label.values(), by_label.keys(), loc="upper right", fontsize="x-small")

def plot_throughput(data, ax, time_filter):
    throughput_df = pd.DataFrame(data['throughput'], columns=['timestamp', 'packet_size', 'direction', 'interface'])
    time_window = 1_000_000_000.0

    if time_filter is not None:
        throughput_df = throughput_df.query(f'timestamp >= {time_filter[0] - time_window} and timestamp <= {time_filter[1] + time_window}')
        throughput_df.loc[:, 'timestamp'] -= time_filter[0]

    # Calculate time bins (floor of timestamp in seconds)
    throughput_df['time_bin'] = (throughput_df['timestamp'] // time_window) * time_window

    # Group by time_bin and sum packet sizes
    throughput_df = throughput_df.groupby(['time_bin', 'direction', 'interface']).agg({
        'packet_size': 'sum',
        'timestamp': 'count'
    }).reset_index()

    # Compute Mbps
    throughput_df['throughput'] = (throughput_df['packet_size'] * 8) / 1_000_000

    if time_filter is not None:
        throughput_df = throughput_df.iloc[1:]

    throughput_df['throughput_smoothed'] = throughput_df['throughput'].rolling(window=3, center=True).mean()

    used_interfaces = sorted(list([interface for interface in dict.fromkeys(throughput_df['interface'])]))

    colormap = cm.get_cmap('tab20b', len(used_interfaces)).reversed()
    interface_colors = {
        iface: colormap(i)
        for i, iface in enumerate(used_interfaces)
    }

    for interface in used_interfaces:
        color = interface_colors[interface]

        for direction in ["Ingress", "Egress"]:
            if direction == "Ingress":
                line_style = '-'
            else:
                line_style = '--'

            interface_df = throughput_df.query(f'interface == "{interface}" and direction == "{direction}"')
            ax.plot(
                interface_df['time_bin'],
                interface_df['throughput'],
                alpha=1,
                marker='o',
                linestyle=line_style,
                label=f'raw {interface} {direction.lower()}',
                color=color
            )

    ax.yaxis.tick_right()
    ax.set_ylabel("Throughput\n(Mbps)")

    ax.plot(throughput_df['time_bin'], throughput_df['throughput_smoothed'], marker='o', linestyle='-', label="Smoothed total", color="black")
    ax.plot(throughput_df['time_bin'], throughput_df['throughput'], alpha=0.25, marker='o', linestyle='-', label="Raw total", color="black")
    ax.grid(True, linestyle='--', alpha=0.7)

    handles, labels = ax.get_legend_handles_labels()
    by_label = dict(zip(labels, handles))
    ax.legend(by_label.values(), by_label.keys(), loc="upper right", fontsize="x-small")

def plot_xdp(data, ax, time_filter, xdp_labels):
    xdp_times_df = pd.DataFrame(data['xdp_times'], columns=['timestamp', 'text'])

    if time_filter is not None:
        xdp_times_df = xdp_times_df.query(f'timestamp >= {time_filter[0]} and timestamp <= {time_filter[1]}')
        xdp_times_df.loc[:, 'timestamp'] -= time_filter[0]

    for i, row in xdp_times_df.iterrows():
        if time_filter is not None and (row['timestamp'] < time_filter[0] or row['timestamp'] > time_filter[1]):
            continue

        ax.axvline(x=row['timestamp'], linestyle="--", linewidth=0.5, color='black', alpha=0.5)

        if xdp_labels:
            text = ax.text(
                row['timestamp'] - 10000,
                0.1,
                row['text'].replace(", ", "\n"),
                ha="center",
                color="black",
                alpha=0.5,
                rotation="vertical",
                size="x-small",
                visible=False
            )

            texts.append((text, 1000000, row['timestamp'] - 500000, row['timestamp'] + 500000))

def main():
    f = open(args.input, 'r')
    data = json.load(f)

    fig = plt.figure()
    plots = {}
    used_plots = []
    used_cpus = []

    if args.stack_merged or args.stack_per_cpu:
        time_df = pd.json_normalize(data['execution_times'], meta=["function_name", "start_time", "end_time", "duration", "inner_duration", "depth", "cpuid"])
        cpuids = dict.fromkeys(time_df["cpuid"])

        if time_filter is not None:
            time_df = time_df.query(f'start_time >= {time_filter[0]} and start_time <= {time_filter[1]}')
            time_df.loc[:, 'start_time'] -= time_filter[0]
            time_df.loc[:, 'end_time'] -= time_filter[0]

        time_df['depth'] = time_df['depth'].apply(lambda x: x/10)
        time_df['inner_duration_perc'] = time_df['inner_duration'] * 100 / time_df['duration']

        if args.stack_merged:
            used_plots.append('stack_merged')
        elif args.stack_per_cpu:
            used_cpus = sorted(list([cpu for cpu in cpuids if str(cpu) not in args.ignored_cpus]))
            for i in used_cpus:
                used_plots.append('stack_per_cpu')

    if args.memory:
        used_plots.append('memory')

    if args.throughput:
        used_plots.append('throughput')

    first_ax = None
    last_ax = None

    for index, plot in enumerate(used_plots):
        ax = fig.add_subplot(len(used_plots), 1, index+1, sharex=first_ax)
        plots[f'{plot}_{index}'] = ax

        if index == 0:
            first_ax = ax
        if index == len(used_plots) - 1:
            last_ax = ax

    for name, ax in plots.items():
        if name.startswith('stack_merged'):
            plot_stack_merged(time_df, ax, time_filter)
        elif name.startswith('stack_per_cpu'):
            ax.sharey = first_ax
            cpuid = used_cpus.pop(0)
            plot_stack_per_cpu(time_df, cpuid, ax, time_filter)
        elif name.startswith('memory'):
            plot_memory(data, ax, time_filter)
        elif name.startswith('throughput'):
            plot_throughput(data, ax, time_filter)

        if args.xdp and name.startswith('stack_merged') or name.startswith('stack_per_cpu'):
            plot_xdp(data, ax, time_filter, args.xdp_labels)

    first_ax.callbacks.connect('xlim_changed', lambda event_ax: update_text_visibility(event_ax, fig, first_ax))
    update_text_visibility(first_ax, fig, first_ax)
    last_ax.set_xlabel("Time (ns)")

    plt.tight_layout()
    plt.subplots_adjust(top=0.945, bottom=0.095, left=0.06, right=0.92, hspace=0.8)
    plt.show()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        prog='FlameLine'
    )

    parser.add_argument('-i', '--input', type=pathlib.Path, default='shared/results.json')
    parser.add_argument('-s', '--stack-merged', action='store_true', default=False)
    parser.add_argument('-s+', '--stack-per-cpu', action='store_true', default=False)
    parser.add_argument('-m', '--memory', action='store_true', default=False)
    parser.add_argument('-t', '--throughput', action='store_true', default=False)
    parser.add_argument('-x', '--xdp', action='store_true', default=False)
    parser.add_argument('-f', '--filter', default=None)
    parser.add_argument('--ignored-cpus', nargs='+', default=[])
    parser.add_argument('--xdp-labels', action='store_true', default=False)

    args = parser.parse_args()

    texts = []

    if args.filter is not None:
        time_filter = args.filter
        time_filter = time_filter.split('-')
        time_filter[0] = int(time_filter[0])
        time_filter[1] = int(time_filter[1])

        if time_filter[1] < time_filter[0]:
            print("Invalid time window")
            exit(1)
    else:
        time_filter = None

    main()