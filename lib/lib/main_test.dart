import 'package:flutter/material.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const MaterialApp(
    title: 'wTools Test',
    home: Center(child: Text('Test', style: TextStyle(fontSize: 32, color: Colors.white))),
  ));
}
