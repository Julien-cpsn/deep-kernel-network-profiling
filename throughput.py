import matplotlib.pyplot as plt
import matplotlib.cm as cm
import pandas as pd
import numpy as np
import math
import json
import sys

if len(sys.argv) > 1:
    file_path = sys.argv[1]
else:
    file_path = 'shared/results.json'

if len(sys.argv) > 2:
    time_filter = sys.argv[2]
    time_filter = time_filter.split('-')
    time_filter[0] = int(time_filter[0])
    time_filter[1] = int(time_filter[1])

    if time_filter[1] < time_filter[0]:
        print("Invalid time window")
        exit(1)
else:
    time_filter = None

f = open(file_path, 'r')
data = json.load(f)

# Create figure and axis
fig = plt.figure()
ax = fig.add_subplot(1, 1, 1)

throughput_df = pd.DataFrame(data['throughput'], columns=['timestamp', 'packet_size', 'direction', 'interface'])
time_window = 1_000_000_000.0

if time_filter is not None:
    throughput_df = throughput_df.query(f'timestamp >= {time_filter[0] - time_window} and timestamp <= {time_filter[1] + time_window}')
    throughput_df['timestamp'] = throughput_df['timestamp'].apply(lambda x: x - time_filter[0])

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

ax.plot(throughput_df['time_bin'], throughput_df['throughput_smoothed'], marker='o', linestyle='-', label="Smoothed total", color="black")
ax.plot(throughput_df['time_bin'], throughput_df['throughput'], alpha=0.25, marker='o', linestyle='-', label="Raw total", color="black")
ax.grid(True, linestyle='--', alpha=0.7)

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
ax.set_xlabel("Time (ns)")
ax.set_ylabel("Throughput (Mbps)")


x_min = throughput_df['time_bin'].min()
x_max = throughput_df['time_bin'].max()
x_step = 1e9
xticks = np.arange(x_min, x_max + x_step, x_step)
ax.set_xlim(x_min, x_max)
ax.set_xticks(xticks)
ax.set_xticklabels([f"{int(t/1e9)}s" for t in xticks])

handles, labels = ax.get_legend_handles_labels()
by_label = dict(zip(labels, handles))
ax.legend(by_label.values(), by_label.keys(), loc="upper left")

plt.tight_layout()
plt.show()