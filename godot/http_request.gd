extends HTTPRequest

 
var headers

signal data_recieved 


func _ready():
	headers = ["Content-Type: application/json",'Accept: application/json']
	# Create an HTTP request node and connect its completion signal.
	request_completed.connect(self._http_request_completed)
	take_turns(3)



func take_turns(turns:int):
	var body = {
		'repeat':turns
	}
	var json_body = JSON.new().stringify(body)
	var error = request("http://127.0.0.1:3000/turn/execute", headers, HTTPClient.METHOD_POST, json_body)
	if error != OK:
		push_error("An error occurred in the HTTP request.")
	
#func get_list_views(space_id, list_id):
	#var error = request(str("http://127.0.0.1:31009/v1/spaces/",space_id, "/lists/",list_id,"/views"), headers, HTTPClient.METHOD_GET)
	#if error != OK:
		#push_error("An error occurred in the HTTP request.")
#
#
#
#func get_list_view_objects(space_id, list_id, view_id):
	#
	#var error = request(str("http://127.0.0.1:31009/v1/spaces/",space_id, "/lists/",list_id,"/views/", view_id, "/objects"), headers, HTTPClient.METHOD_GET)
	#if error != OK:
		#push_error("An error occurred in the HTTP request.")
#






func _http_request_completed(result, response_code, headers, body):
	var json = JSON.new()
	json.parse(body.get_string_from_utf8())
	var response = json.get_data()

	data_recieved.emit(response)
	print(response)
