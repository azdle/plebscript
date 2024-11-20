local response = http.request {
	url = 'http://www.random.org/integers/',
	params = {
		num=1, min=0, max=1,
		format='plain', rnd='new',
		col=1, base=10
	}
}
if tonumber(response.content) == 0 then
	return 'heads'
else
	return 'tails'
end
