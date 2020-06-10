# use with https://github.com/casey/just

# Be inspired to use just to notify a chat
# channel, this examples shows use with keybase
# since it - practically - authenticates at the
# device level and needs no additional secrets

# notify update in keybase
notify m="":
	keybase chat send --topic-type "chat" --channel <channel> <team> "upd(<repo>): {{m}}"
