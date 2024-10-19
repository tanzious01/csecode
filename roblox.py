from operator import index

import pandas as pd
from numpy import nan

import rblx_py

df = pd.read_json("roblox.json")
df = df.reset_index()
df = df.rename(columns={"index": "ids"})
slice = df[df["user_friends"].isna()]["ids"].tolist()
