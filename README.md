# Substrate Node Template

[![Try on playground](https://img.shields.io/badge/Playground-Node_Template-brightgreen?logo=Parity%20Substrate)](https://docs.substrate.io/playground/) [![Matrix](https://img.shields.io/matrix/substrate-technical:matrix.org)](https://matrix.to/#/#substrate-technical:matrix.org)

1.	Upload File

a.	Application will need to store all the details based on the inputs
b.	Owner of the ﬁle is the account id of the uploader.
c.	Users will need to only pass the link of the ﬁle. 
d.	Must take inputs:
i.	ﬁle_cid(content_id)
ii.	FIle_link
iii.	allow_download: bool
iv.	File_type:
v.	cost (only for Privileged ﬁles)
vi.	ﬁle_size:

2.	Download File

a.	Update the storages accordingly as per the extrinsic completion
b.	Only ﬁles that have allow_download as true can be downloaded
c.	If ﬁle is privileged user will need to pay the cost for that ﬁle and won't need to pay  for excess ﬁle size if exists
d.	Inputs:
i.	User_accountid
ii.	ﬁle_cid

3.	Transfer Owner

a.	Can only be called by the ﬁle owner
b.	Inputs:
i.	FIle_cid
ii.	new_owner_account_id

1.	Storage that stores ﬁle information based on users
2.	Number of downloads per ﬁle
3.	Ensure statements where ever necessary
